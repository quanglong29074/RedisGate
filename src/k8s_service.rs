// Kubernetes service for Redis instance management

use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{
    Container, ContainerPort, EnvVar, PodSpec, PodTemplateSpec, Secret, Service, ServicePort, ServiceSpec,
};
use k8s_openapi::api::networking::v1::{Ingress, IngressBackend, IngressRule, IngressServiceBackend, IngressSpec, HTTPIngressPath, HTTPIngressRuleValue};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::{Api, Client, Error as KubeError};
use std::collections::BTreeMap;
use uuid::Uuid;

pub struct K8sRedisService {
    client: Client,
}

#[derive(Debug)]
pub struct RedisDeploymentConfig {
    pub name: String,
    pub slug: String,
    pub namespace: String,
    pub organization_id: Uuid,
    pub instance_id: Uuid,
    pub redis_version: String,
    pub max_memory: i64,
    pub redis_password: String,
    pub port: i32,
}

#[derive(Debug)]
pub struct K8sDeploymentResult {
    pub deployment_name: String,
    pub service_name: String,
    pub ingress_name: String,
    pub namespace: String,
    pub port: i32,
    pub domain: String,
}

impl K8sRedisService {
    pub async fn new() -> Result<Self, KubeError> {
        let client = Client::try_default().await?;
        Ok(Self { client })
    }

    /// Create a complete Redis deployment with service and ingress
    pub async fn create_redis_instance(
        &self,
        config: RedisDeploymentConfig,
    ) -> Result<K8sDeploymentResult, KubeError> {
        let deployment_name = format!("redis-{}", config.slug);
        let service_name = format!("redis-{}-service", config.slug);
        let ingress_name = format!("redis-{}-ingress", config.slug);
        let domain = format!("{}.{}.redis.local", config.slug, config.organization_id.simple());

        // Create namespace if it doesn't exist
        self.ensure_namespace(&config.namespace).await?;

        // Create Redis secret for password
        self.create_redis_secret(&config).await?;

        // Create Redis deployment
        self.create_redis_deployment(&config).await?;

        // Create Redis service
        self.create_redis_service(&config).await?;

        // Create Redis ingress for TCP port exposure
        self.create_redis_ingress(&config).await?;

        Ok(K8sDeploymentResult {
            deployment_name,
            service_name,
            ingress_name,
            namespace: config.namespace,
            port: config.port,
            domain,
        })
    }

    /// Delete a Redis deployment and all related resources
    pub async fn delete_redis_instance(
        &self,
        namespace: &str,
        slug: &str,
    ) -> Result<(), KubeError> {
        let deployment_name = format!("redis-{}", slug);
        let service_name = format!("redis-{}-service", slug);
        let ingress_name = format!("redis-{}-ingress", slug);
        let secret_name = format!("redis-{}-secret", slug);

        // Delete ingress
        let ingresses: Api<Ingress> = Api::namespaced(self.client.clone(), namespace);
        let _ = ingresses.delete(&ingress_name, &Default::default()).await;

        // Delete service
        let services: Api<Service> = Api::namespaced(self.client.clone(), namespace);
        let _ = services.delete(&service_name, &Default::default()).await;

        // Delete deployment
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), namespace);
        let _ = deployments.delete(&deployment_name, &Default::default()).await;

        // Delete secret
        let secrets: Api<Secret> = Api::namespaced(self.client.clone(), namespace);
        let _ = secrets.delete(&secret_name, &Default::default()).await;

        Ok(())
    }

    /// Check deployment status
    pub async fn get_deployment_status(
        &self,
        namespace: &str,
        slug: &str,
    ) -> Result<String, KubeError> {
        let deployment_name = format!("redis-{}", slug);
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), namespace);
        
        match deployments.get(&deployment_name).await {
            Ok(deployment) => {
                if let Some(status) = deployment.status {
                    if let Some(ready_replicas) = status.ready_replicas {
                        if ready_replicas > 0 {
                            return Ok("running".to_string());
                        }
                    }
                    if let Some(replicas) = status.replicas {
                        if replicas > 0 {
                            return Ok("pending".to_string());
                        }
                    }
                }
                Ok("unknown".to_string())
            }
            Err(_) => Ok("failed".to_string()),
        }
    }

    async fn ensure_namespace(&self, namespace: &str) -> Result<(), KubeError> {
        use k8s_openapi::api::core::v1::Namespace;
        
        let namespaces: Api<Namespace> = Api::all(self.client.clone());
        
        // Try to get the namespace first
        if namespaces.get(namespace).await.is_ok() {
            return Ok(());
        }

        // Create namespace if it doesn't exist
        let ns = Namespace {
            metadata: ObjectMeta {
                name: Some(namespace.to_string()),
                labels: Some({
                    let mut labels = BTreeMap::new();
                    labels.insert("created-by".to_string(), "redisgate".to_string());
                    labels
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        namespaces.create(&Default::default(), &ns).await?;
        Ok(())
    }

    async fn create_redis_secret(&self, config: &RedisDeploymentConfig) -> Result<(), KubeError> {
        let secret_name = format!("redis-{}-secret", config.slug);
        let secrets: Api<Secret> = Api::namespaced(self.client.clone(), &config.namespace);

        let mut string_data = BTreeMap::new();
        string_data.insert("redis-password".to_string(), config.redis_password.clone());

        let secret = Secret {
            metadata: ObjectMeta {
                name: Some(secret_name),
                namespace: Some(config.namespace.clone()),
                labels: Some({
                    let mut labels = BTreeMap::new();
                    labels.insert("app".to_string(), format!("redis-{}", config.slug));
                    labels.insert("created-by".to_string(), "redisgate".to_string());
                    labels.insert("instance-id".to_string(), config.instance_id.to_string());
                    labels
                }),
                ..Default::default()
            },
            string_data: Some(string_data),
            ..Default::default()
        };

        secrets.create(&Default::default(), &secret).await?;
        Ok(())
    }

    async fn create_redis_deployment(&self, config: &RedisDeploymentConfig) -> Result<(), KubeError> {
        let deployment_name = format!("redis-{}", config.slug);
        let secret_name = format!("redis-{}-secret", config.slug);
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), &config.namespace);

        let memory_limit = format!("{}Mi", config.max_memory / (1024 * 1024)); // Convert bytes to Mi
        let memory_request = format!("{}Mi", std::cmp::max(64, config.max_memory / (1024 * 1024) / 2)); // At least 64Mi, half of limit

        let deployment = Deployment {
            metadata: ObjectMeta {
                name: Some(deployment_name.clone()),
                namespace: Some(config.namespace.clone()),
                labels: Some({
                    let mut labels = BTreeMap::new();
                    labels.insert("app".to_string(), format!("redis-{}", config.slug));
                    labels.insert("created-by".to_string(), "redisgate".to_string());
                    labels.insert("instance-id".to_string(), config.instance_id.to_string());
                    labels
                }),
                ..Default::default()
            },
            spec: Some(DeploymentSpec {
                replicas: Some(1),
                selector: LabelSelector {
                    match_labels: Some({
                        let mut labels = BTreeMap::new();
                        labels.insert("app".to_string(), format!("redis-{}", config.slug));
                        labels
                    }),
                    ..Default::default()
                },
                template: PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some({
                            let mut labels = BTreeMap::new();
                            labels.insert("app".to_string(), format!("redis-{}", config.slug));
                            labels.insert("created-by".to_string(), "redisgate".to_string());
                            labels
                        }),
                        ..Default::default()
                    }),
                    spec: Some(PodSpec {
                        containers: vec![Container {
                            name: "redis".to_string(),
                            image: Some(format!("redis:{}", config.redis_version)),
                            ports: Some(vec![ContainerPort {
                                container_port: config.port,
                                name: Some("redis".to_string()),
                                protocol: Some("TCP".to_string()),
                                ..Default::default()
                            }]),
                            env: Some(vec![
                                EnvVar {
                                    name: "REDIS_PASSWORD".to_string(),
                                    value_from: Some(k8s_openapi::api::core::v1::EnvVarSource {
                                        secret_key_ref: Some(k8s_openapi::api::core::v1::SecretKeySelector {
                                            name: Some(secret_name),
                                            key: "redis-password".to_string(),
                                            optional: Some(false),
                                        }),
                                        ..Default::default()
                                    }),
                                    ..Default::default()
                                },
                            ]),
                            command: Some(vec![
                                "redis-server".to_string(),
                                "--requirepass".to_string(),
                                "$(REDIS_PASSWORD)".to_string(),
                                "--maxmemory".to_string(),
                                format!("{}b", config.max_memory),
                                "--maxmemory-policy".to_string(),
                                "allkeys-lru".to_string(),
                            ]),
                            resources: Some(k8s_openapi::api::core::v1::ResourceRequirements {
                                limits: Some({
                                    let mut limits = BTreeMap::new();
                                    limits.insert("memory".to_string(), k8s_openapi::apimachinery::pkg::api::resource::Quantity(memory_limit.clone()));
                                    limits.insert("cpu".to_string(), k8s_openapi::apimachinery::pkg::api::resource::Quantity("500m".to_string()));
                                    limits
                                }),
                                requests: Some({
                                    let mut requests = BTreeMap::new();
                                    requests.insert("memory".to_string(), k8s_openapi::apimachinery::pkg::api::resource::Quantity(memory_request));
                                    requests.insert("cpu".to_string(), k8s_openapi::apimachinery::pkg::api::resource::Quantity("100m".to_string()));
                                    requests
                                }),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                },
                ..Default::default()
            }),
            ..Default::default()
        };

        deployments.create(&Default::default(), &deployment).await?;
        Ok(())
    }

    async fn create_redis_service(&self, config: &RedisDeploymentConfig) -> Result<(), KubeError> {
        let service_name = format!("redis-{}-service", config.slug);
        let services: Api<Service> = Api::namespaced(self.client.clone(), &config.namespace);

        let service = Service {
            metadata: ObjectMeta {
                name: Some(service_name.clone()),
                namespace: Some(config.namespace.clone()),
                labels: Some({
                    let mut labels = BTreeMap::new();
                    labels.insert("app".to_string(), format!("redis-{}", config.slug));
                    labels.insert("created-by".to_string(), "redisgate".to_string());
                    labels.insert("instance-id".to_string(), config.instance_id.to_string());
                    labels
                }),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                selector: Some({
                    let mut selector = BTreeMap::new();
                    selector.insert("app".to_string(), format!("redis-{}", config.slug));
                    selector
                }),
                ports: Some(vec![ServicePort {
                    name: Some("redis".to_string()),
                    port: config.port,
                    target_port: Some(IntOrString::Int(config.port)),
                    protocol: Some("TCP".to_string()),
                    ..Default::default()
                }]),
                type_: Some("ClusterIP".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        services.create(&Default::default(), &service).await?;
        Ok(())
    }

    async fn create_redis_ingress(&self, config: &RedisDeploymentConfig) -> Result<(), KubeError> {
        let ingress_name = format!("redis-{}-ingress", config.slug);
        let service_name = format!("redis-{}-service", config.slug);
        let domain = format!("{}.{}.redis.local", config.slug, config.organization_id.simple());
        
        let ingresses: Api<Ingress> = Api::namespaced(self.client.clone(), &config.namespace);

        let ingress = Ingress {
            metadata: ObjectMeta {
                name: Some(ingress_name.clone()),
                namespace: Some(config.namespace.clone()),
                labels: Some({
                    let mut labels = BTreeMap::new();
                    labels.insert("app".to_string(), format!("redis-{}", config.slug));
                    labels.insert("created-by".to_string(), "redisgate".to_string());
                    labels.insert("instance-id".to_string(), config.instance_id.to_string());
                    labels
                }),
                annotations: Some({
                    let mut annotations = BTreeMap::new();
                    annotations.insert("nginx.ingress.kubernetes.io/tcp-services-configmap".to_string(), 
                                     format!("{}/{}", config.namespace, "tcp-services"));
                    annotations.insert("kubernetes.io/ingress.class".to_string(), "nginx".to_string());
                    annotations
                }),
                ..Default::default()
            },
            spec: Some(IngressSpec {
                rules: Some(vec![IngressRule {
                    host: Some(domain.clone()),
                    http: Some(HTTPIngressRuleValue {
                        paths: vec![HTTPIngressPath {
                            path: Some("/".to_string()),
                            path_type: "Prefix".to_string(),
                            backend: IngressBackend {
                                service: Some(IngressServiceBackend {
                                    name: service_name,
                                    port: Some(k8s_openapi::api::networking::v1::ServiceBackendPort {
                                        number: Some(config.port),
                                        ..Default::default()
                                    }),
                                }),
                                ..Default::default()
                            },
                        }],
                    }),
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };

        ingresses.create(&Default::default(), &ingress).await?;
        Ok(())
    }
}