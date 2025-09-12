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

        println!("🔄 Đang khởi tạo RedisPoolManager với cấu hình: {:?}", config);

        Ok(Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            config: config.clone(),
            k8s_client,
        })
    }

    // Kiểm tra xem có đang chạy trong cluster không
    fn is_running_in_cluster(&self) -> bool {
        std::env::var("KUBERNETES_SERVICE_HOST").is_ok()
    }

    // lấy kết nối Redis từ pool đã có, nếu ko có pool hoặc ko lấy đc pool thì tạo pool mới.
    pub async fn get_client(&self, instance_name: &str) -> Option<deadpool_redis::Connection> {
        println!("➡️ Đang lấy pool cho instance: {}", instance_name);

        // Try to get existing pool
        if let Some(pool) = self.get_pool(instance_name).await {
            println!("🟢 Tìm thấy pool cho instance {}", instance_name);
            match pool.get().await {
                Ok(conn) => {
                    println!("✅ Lấy được kết nối Redis từ pool cho instance {}", instance_name);
                    return Some(conn);
                }
                Err(e) => {
                    tracing::warn!("Failed to get connection from pool for {}: {}", instance_name, e);
                    println!("❌ Không thể lấy kết nối từ pool cho instance {}: {}", instance_name, e);
                    // Pool might be stale, remove it and try to recreate
                    self.remove_pool(instance_name).await;
                    println!("🔄 Đã xóa pool cũ cho instance {}", instance_name);
                }
            }
        } else {
            println!("❌ Không tìm thấy pool cho instance {}", instance_name);
        }

        // Try to create new pool for this instance
        println!("🆕 Đang tạo pool mới cho instance {}", instance_name);
        if let Ok(pool) = self.create_pool_for_instance(instance_name).await {
            self.add_pool(instance_name.to_string(), pool.clone()).await;
            println!("✅ Đã thêm pool mới cho instance {}", instance_name);
            pool.get().await.ok()
        } else {
            println!("❌ Không thể tạo pool mới cho instance {}", instance_name);
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
        println!("🆕 Tạo pool mới cho instance: {}", instance_name);

        let mut pools = self.pools.write().await;
        pools.insert(instance_name, pool);
    }

    // xoá pool Redis
    async fn remove_pool(&self, instance_name: &str) {
        println!("🔄 Đang xoá pool cho instance: {}", instance_name);
        let mut pools = self.pools.write().await;
        pools.remove(instance_name);
    }

    // tạo ra một pool kết nối Redis mới cho một Redis instance
    async fn create_pool_for_instance(&self, instance_name: &str) -> Result<Pool> {
        // Kiểm tra xem có đang chạy trong cluster không
        let redis_url = if self.is_running_in_cluster() {
            println!("🏠 Đang chạy trong Kubernetes cluster - sử dụng service discovery");
            self.discover_redis_service(instance_name).await?
        } else {
            println!("🖥️ Đang chạy ngoài cluster - sử dụng localhost với port-forward");
            self.create_localhost_url(instance_name).await?
        };

        println!("🔧 Đang tạo pool Redis cho instance {} tại URL: {}", instance_name, redis_url);

        // Create manager from Redis URL
        let manager = deadpool_redis::Manager::new(redis_url.as_str())?;

        let mut config = deadpool_redis::Config::from_url(&redis_url);

        // Tạo PoolConfig nếu chưa có
        let mut pool_cfg = config.get_pool_config();
        pool_cfg.max_size = self.config.pool_max_size;
        pool_cfg.timeouts.wait = Some(Duration::from_secs(self.config.pool_timeout_seconds));
        config.pool = Some(pool_cfg);

        // Tạo pool
        let pool = match config.create_pool(Some(Runtime::Tokio1)) {
            Ok(p) => p,
            Err(e) => {
                println!("❌ Lỗi khi tạo deadpool_redis::Pool cho instance {}: {}", instance_name, e);
                return Err(e.into());
            }
        };

        // Test the connection
        let mut conn = match pool.get().await {
            Ok(c) => c,
            Err(e) => {
                println!("❌ Lỗi khi lấy kết nối từ pool vừa tạo cho instance {}: {}", instance_name, e);
                return Err(e.into());
            }
        };

        match redis::cmd("PING").query_async::<_, String>(conn.as_mut()).await {
            Ok(pong) => {
                println!("✅ Đã kết nối với Redis {}: PONG={}", instance_name, pong);
            }
            Err(e) => {
                println!("❌ Không thể PING Redis {}: {}", instance_name, e);
                return Err(e.into());
            }
        }

        Ok(pool as deadpool_redis::Pool)
    }

    // Tạo URL cho localhost (khi sử dụng port-forward)
    async fn create_localhost_url(&self, instance_name: &str) -> Result<String> {
        println!("🔗 Tạo localhost URL cho instance: {}", instance_name);

        // Mapping instance names to local ports
        // Bạn có thể mở rộng logic này để support nhiều instances
        let port = match instance_name {
            "my-redis-replicas" => 6379,
            "my-redis-master" => 6380, // Nếu bạn port-forward master to 6380
            _ => {
                println!("❌ Không hỗ trợ instance: {}", instance_name);
                return Err(crate::error::GatewayError::InstanceNotFound(instance_name.to_string()));
            }
        };

        let mut url = format!("redis://localhost:{}", port);

        if let Some(password) = &self.config.default_password {
            println!("🔑 Sử dụng password cho localhost connection");
            url = format!("redis://:{}@localhost:{}", password, port);
        }

        println!("✅ Localhost URL: {}", url);
        Ok(url)
    }

    // khám phá dịch vụ Redis trong Kubernetes dựa trên tên instance
    async fn discover_redis_service(&self, instance_name: &str) -> Result<String> {
        println!("🔍 Đang truy vấn service: {}", instance_name);

        use kube::{Api, api::ListParams};
        use k8s_openapi::api::core::v1::Service;

        let services: Api<Service> = Api::default_namespaced(self.k8s_client.clone());

        println!("🔍 Trying to get service: {}", instance_name);
        // Lấy service theo tên
        let service = services.get(instance_name).await?;
        println!("✅ Đã lấy được service: {:?}", service);

        println!("✅ Đã lấy được service name: {:?}", service.metadata.name);
        println!("✅ Đã lấy được service namespace: {:?}", service.metadata.namespace);

        if let (Some(name), Some(namespace)) = (&service.metadata.name, &service.metadata.namespace) {
            let host = format!("{}.{}.svc.cluster.local", name, namespace);
            let port = service.spec
                .as_ref()
                .and_then(|spec| spec.ports.as_ref())
                .and_then(|ports| ports.first())
                .map(|port| port.port)
                .unwrap_or(6379);

            let mut url = format!("redis://{}:{}", host, port);

            if let Some(password) = &self.config.default_password {
                println!("🔑 Password lấy từ Secret ({}): {}", instance_name, password);
                url = format!("redis://:{}@{}:{}", password, host, port);
            }

            return Ok(url);
        }
        println!("❌ Không tìm thấy instance: {}", instance_name);

        println!("InstanceNotFound");
        Err(crate::error::GatewayError::InstanceNotFound(instance_name.to_string()))
    }

    pub async fn refresh_pools(&self) -> Result<()> {
        // Nếu đang chạy ngoài cluster, skip refresh
        if !self.is_running_in_cluster() {
            println!("🏠 Đang chạy ngoài cluster - bỏ qua refresh pools");
            return Ok(());
        }

        // Discover all Redis instances
        let instances = self.discover_all_instances().await?;

        println!("🔄 Đang làm mới pools cho tất cả các instance Redis");

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
                println!("❌ Đã xóa pool cho instance không còn tồn tại: {}", pool_name);
            }
        }

        // Create pools for new instances
        for instance in &instances {
            if !current_pools.contains(instance) {
                tracing::info!("Creating pool for new instance: {}", instance);
                println!("🆕 Đang tạo pool cho instance mới: {}", instance);
                if let Ok(pool) = self.create_pool_for_instance(instance).await {
                    self.add_pool(instance.clone(), pool).await;
                    println!("✅ Đã tạo pool cho instance mới: {}", instance);
                } else {
                    println!("❌ Không thể tạo pool cho instance mới: {}", instance);
                }
            }
        }

        Ok(())
    }

    // khám phá tất cả các instance Redis đang chạy trong Kubernetes
    async fn discover_all_instances(&self) -> Result<Vec<String>> {
        use kube::{Api, api::ListParams};
        use k8s_openapi::api::core::v1::Service;

        println!("🔍 Đang khám phá tất cả các Redis instances");

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

        println!("✅ Đã khám phá các instance Redis: {:?}", instances);

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