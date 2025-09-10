// src/redis/pool.rs
use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use tokio::sync::RwLock;
use redis::{Client, Connection};
use deadpool_redis::{Config as PoolConfig, Pool, Runtime};

use crate::{config::RedisConfig, error::Result};

pub struct RedisPoolManager {
    // các pool Redis được phân chia theo tên của các instance Redis.
    pools: Arc<RwLock<HashMap<String, Pool>>>,
    // Cấu hình Redis
    config: RedisConfig,
    // Đây là client Kubernetes
    k8s_client: kube::Client,
}

impl RedisPoolManager {
    // Tạo một đối tượng RedisPoolManager mới, khởi tạo client k8s, pools, config
    pub async fn new(config: &RedisConfig) -> Result<Self> {
        let k8s_client = kube::Client::try_default().await?;

        Ok(Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            config: config.clone(),
            k8s_client,
        })
    }

    // lấy kết nối Redis từ pool đã có, nếu ko có pool hoặc ko lấy đc pool thì tạo pool mới.
    pub async fn get_client(&self, instance_name: &str) -> Option<deadpool_redis::Connection> {
        // Try to get existing pool
        if let Some(pool) = self.get_pool(instance_name).await {
            match pool.get().await {
                Ok(conn) => return Some(conn),
                Err(e) => {
                    tracing::warn!("Failed to get connection from pool for {}: {}", instance_name, e);
                    // Pool might be stale, remove it and try to recreate
                    self.remove_pool(instance_name).await;
                }
            }
        }

        // Try to create new pool for this instance
        if let Ok(pool) = self.create_pool_for_instance(instance_name).await {
            self.add_pool(instance_name.to_string(), pool.clone()).await;
            pool.get().await.ok()
        } else {
            None
        }
    }

    // Lấy pool Redis của một instance cụ thể từ pools
    async fn get_pool(&self, instance_name: &str) -> Option<Pool> {
        let pools = self.pools.read().await;
        pools.get(instance_name).cloned()
    }


    // Thêm một pool Redis vào pools khi đã tạo được pool mới.
    async fn add_pool(&self, instance_name: String, pool: Pool) {
        let mut pools = self.pools.write().await;
        pools.insert(instance_name, pool);
    }

    // xoá pool Redis
    async fn remove_pool(&self, instance_name: &str) {
        let mut pools = self.pools.write().await;
        pools.remove(instance_name);
    }

    // tạo ra một pool kết nối Redis mới cho một Redis instance
    async fn create_pool_for_instance(&self, instance_name: &str) -> Result<Pool> {
        // Discover Redis service in Kubernetes
        let redis_url = self.discover_redis_service(instance_name).await?;

        tracing::info!("Creating Redis pool for instance {} at {}", instance_name, redis_url);

        // Create manager from Redis URL
        let manager = deadpool_redis::Manager::new(redis_url.as_str())?;


        let mut config = deadpool_redis::Config::from_url(&redis_url);

        // Tạo PoolConfig nếu chưa có
        let mut pool_cfg = config.get_pool_config();
        pool_cfg.max_size = self.config.pool_max_size;
        pool_cfg.timeouts.wait = Some(Duration::from_secs(self.config.pool_timeout_seconds));
        config.pool = Some(pool_cfg);

        // Tạo pool
        let pool = config.create_pool(Some(Runtime::Tokio1))?;

        // Test the connection
        let mut conn = pool.get().await?;
        redis::cmd("PING")
            .query_async::<_, String>(conn.as_mut()) // <- sửa ở đây
            .await?;

        Ok(pool as deadpool_redis::Pool)

    }


    // khám phá dịch vụ Redis trong Kubernetes dựa trên tên instance
    async fn discover_redis_service(&self, instance_name: &str) -> Result<String> {
        use kube::{Api, api::ListParams};
        use k8s_openapi::api::core::v1::Service;

        let services: Api<Service> = Api::default_namespaced(self.k8s_client.clone());
        let lp = ListParams::default().labels(&format!("instance={}", instance_name));

        let service_list = services.list(&lp).await?;

        if let Some(service) = service_list.items.first() {
            if let (Some(name), Some(namespace)) = (&service.metadata.name, &service.metadata.namespace) {
                // Construct internal Kubernetes DNS name
                let host = format!("{}.{}.svc.cluster.local", name, namespace);
                let port = service.spec
                    .as_ref()
                    .and_then(|spec| spec.ports.as_ref())
                    .and_then(|ports| ports.first())
                    .map(|port| port.port)
                    .unwrap_or(6379);

                let mut url = format!("redis://{}:{}", host, port);

                // Add authentication if configured
                if let Some(password) = &self.config.default_password {
                    url = format!("redis://:{}@{}:{}", password, host, port);
                }

                return Ok(url);
            }
        }

        Err(crate::error::GatewayError::InstanceNotFound(instance_name.to_string()))
    }

    pub async fn refresh_pools(&self) -> Result<()> {
        // Discover all Redis instances
        let instances = self.discover_all_instances().await?;

        // Get current pools
        let current_pools: Vec<String> = {
            let pools = self.pools.read().await;
            pools.keys().cloned().collect()
        };

        // Remove pools for instances that no longer exist
        for pool_name in &current_pools {
            if !instances.contains(pool_name) {
                tracing::info!("Removing pool for deleted instance: {}", pool_name);
                self.remove_pool(pool_name).await;
            }
        }

        // Create pools for new instances
        for instance in &instances {
            if !current_pools.contains(instance) {
                tracing::info!("Creating pool for new instance: {}", instance);
                if let Ok(pool) = self.create_pool_for_instance(instance).await {
                    self.add_pool(instance.clone(), pool).await;
                }
            }
        }

        Ok(())
    }

    // khám phá tất cả các instance Redis đang chạy trong Kubernetes
    async fn discover_all_instances(&self) -> Result<Vec<String>> {
        use kube::{Api, api::ListParams};
        use k8s_openapi::api::core::v1::Service;

        let services: Api<Service> = Api::default_namespaced(self.k8s_client.clone());
        let lp = ListParams::default().labels("app=redis");

        let service_list = services.list(&lp).await?;

        let instances: Vec<String> = service_list
            .items
            .iter()
            .filter_map(|service| {
                service.metadata.labels.as_ref()
                    .and_then(|labels| labels.get("instance"))
                    .cloned()
            })
            .collect();

        Ok(instances)
    }

    // kiểm tra tình trạng (health) của tất cả các pool hiện có
    pub async fn health_check(&self) -> HashMap<String, bool> {
        let pools = self.pools.read().await;
        let mut results = HashMap::new();

        for (instance_name, pool) in pools.iter() {
            match pool.get().await {
                Ok(mut conn) => {
                    match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
                        Ok(_) => results.insert(instance_name.clone(), true),
                        Err(_) => results.insert(instance_name.clone(), false),
                    };
                }
                Err(_) => {
                    results.insert(instance_name.clone(), false);
                }
            }
        }

        results
    }
}

impl Clone for RedisPoolManager {
    fn clone(&self) -> Self {
        Self {
            pools: Arc::clone(&self.pools),
            config: self.config.clone(),
            k8s_client: self.k8s_client.clone(),
        }
    }
}