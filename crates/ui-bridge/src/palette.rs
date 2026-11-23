//! Draw.io + React Flow stencil shape palette catalog and searchable accordion categories.

use rustflow_core::{Node, NodeId, Point, Rect, ShapeKind, SymbolIcon};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StencilCategory {
    Flowchart,
    Aws,
    Gcp,
    Azure,
    InfraDevOps,
    Standard,
    Uml,
    Bpmn,
    CloudArchitecture,
    Network,
    Wireframes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StencilItem {
    pub id: String,
    pub label: String,
    pub category: StencilCategory,
    pub subgroup: String,
    pub shape: ShapeKind,
    pub icon: Option<SymbolIcon>,
    pub default_width: f32,
    pub default_height: f32,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PaletteCatalog {
    items: Vec<StencilItem>,
}

impl Default for PaletteCatalog {
    fn default() -> Self {
        Self::built_in()
    }
}

fn shape_item(
    id: &str,
    label: &str,
    category: StencilCategory,
    shape: ShapeKind,
    w: f32,
    h: f32,
    tags: &[&str],
) -> StencilItem {
    StencilItem {
        id: id.into(),
        label: label.into(),
        category,
        subgroup: String::new(),
        shape,
        icon: None,
        default_width: w,
        default_height: h,
        tags: tags.iter().map(|s| (*s).into()).collect(),
    }
}

fn icon_item(
    id: &str,
    label: &str,
    category: StencilCategory,
    subgroup: &str,
    icon: SymbolIcon,
    tags: &[&str],
) -> StencilItem {
    StencilItem {
        id: id.into(),
        label: label.into(),
        category,
        subgroup: subgroup.into(),
        shape: ShapeKind::RoundedRect,
        icon: Some(icon),
        default_width: 104.0,
        default_height: 88.0,
        tags: tags.iter().map(|s| (*s).into()).collect(),
    }
}

impl PaletteCatalog {
    pub fn built_in() -> Self {
        let items = vec![
            // AWS — Compute & Containers
            icon_item("aws.ec2", "EC2", StencilCategory::Aws, "Compute", SymbolIcon::AwsEc2, &["aws", "compute", "ec2", "vm", "instance"]),
            icon_item("aws.lambda", "Lambda", StencilCategory::Aws, "Compute", SymbolIcon::AwsLambda, &["aws", "compute", "lambda", "serverless", "function"]),
            icon_item("aws.ecs", "ECS", StencilCategory::Aws, "Compute", SymbolIcon::AwsEcs, &["aws", "compute", "ecs", "container", "docker"]),
            icon_item("aws.eks", "EKS", StencilCategory::Aws, "Compute", SymbolIcon::AwsEks, &["aws", "compute", "eks", "kubernetes", "k8s"]),
            icon_item("aws.ecr", "ECR", StencilCategory::Aws, "Compute", SymbolIcon::AwsEcr, &["aws", "compute", "ecr", "registry", "docker", "container"]),
            icon_item("aws.fargate", "Fargate", StencilCategory::Aws, "Compute", SymbolIcon::AwsFargate, &["aws", "compute", "fargate", "serverless", "container"]),
            icon_item("aws.apprunner", "App Runner", StencilCategory::Aws, "Compute", SymbolIcon::AwsAppRunner, &["aws", "compute", "apprunner", "container", "web"]),
            icon_item("aws.beanstalk", "Beanstalk", StencilCategory::Aws, "Compute", SymbolIcon::AwsElasticBeanstalk, &["aws", "compute", "elastic", "beanstalk", "paas"]),
            icon_item("aws.lightsail", "Lightsail", StencilCategory::Aws, "Compute", SymbolIcon::AwsLightsail, &["aws", "compute", "lightsail", "vps"]),
            icon_item("aws.batch", "AWS Batch", StencilCategory::Aws, "Compute", SymbolIcon::AwsBatch, &["aws", "compute", "batch", "jobs"]),
            icon_item("aws.outposts", "Outposts", StencilCategory::Aws, "Compute", SymbolIcon::AwsOutposts, &["aws", "compute", "outposts", "hybrid", "edge"]),
            // AWS — Storage
            icon_item("aws.s3", "S3", StencilCategory::Aws, "Storage", SymbolIcon::AwsS3, &["aws", "storage", "s3", "bucket", "object"]),
            icon_item("aws.ebs", "EBS", StencilCategory::Aws, "Storage", SymbolIcon::AwsEbs, &["aws", "storage", "ebs", "block", "volume"]),
            icon_item("aws.efs", "EFS", StencilCategory::Aws, "Storage", SymbolIcon::AwsEfs, &["aws", "storage", "efs", "nfs", "file"]),
            icon_item("aws.glacier", "S3 Glacier", StencilCategory::Aws, "Storage", SymbolIcon::AwsGlacier, &["aws", "storage", "glacier", "archive", "cold"]),
            icon_item("aws.backup", "AWS Backup", StencilCategory::Aws, "Storage", SymbolIcon::AwsBackup, &["aws", "storage", "backup", "snapshot"]),
            // AWS — Database
            icon_item("aws.rds", "RDS", StencilCategory::Aws, "Database", SymbolIcon::AwsRds, &["aws", "database", "rds", "sql", "postgres", "mysql"]),
            icon_item("aws.dynamodb", "DynamoDB", StencilCategory::Aws, "Database", SymbolIcon::AwsDynamoDb, &["aws", "database", "dynamodb", "nosql", "kv"]),
            icon_item("aws.aurora", "Aurora", StencilCategory::Aws, "Database", SymbolIcon::AwsAurora, &["aws", "database", "aurora", "sql"]),
            icon_item("aws.elasticache", "ElastiCache", StencilCategory::Aws, "Database", SymbolIcon::AwsElastiCache, &["aws", "database", "elasticache", "redis", "memcached"]),
            icon_item("aws.memorydb", "MemoryDB", StencilCategory::Aws, "Database", SymbolIcon::AwsMemoryDb, &["aws", "database", "memorydb", "redis", "valkey"]),
            icon_item("aws.redshift", "Redshift", StencilCategory::Aws, "Database", SymbolIcon::AwsRedshift, &["aws", "database", "redshift", "warehouse", "olap"]),
            icon_item("aws.documentdb", "DocumentDB", StencilCategory::Aws, "Database", SymbolIcon::AwsDocumentDb, &["aws", "database", "documentdb", "mongo", "nosql"]),
            icon_item("aws.neptune", "Neptune", StencilCategory::Aws, "Database", SymbolIcon::AwsNeptune, &["aws", "database", "neptune", "graph"]),
            icon_item("aws.timestream", "Timestream", StencilCategory::Aws, "Database", SymbolIcon::AwsTimestream, &["aws", "database", "timestream", "timeseries", "iot"]),
            icon_item("aws.keyspaces", "Keyspaces", StencilCategory::Aws, "Database", SymbolIcon::AwsKeyspaces, &["aws", "database", "keyspaces", "cassandra"]),
            // AWS — Networking
            icon_item("aws.vpc", "VPC", StencilCategory::Aws, "Network", SymbolIcon::AwsVpc, &["aws", "network", "vpc", "subnet"]),
            icon_item("aws.apigw", "API Gateway", StencilCategory::Aws, "Network", SymbolIcon::AwsApiGateway, &["aws", "network", "api", "gateway", "rest"]),
            icon_item("aws.cloudfront", "CloudFront", StencilCategory::Aws, "Network", SymbolIcon::AwsCloudFront, &["aws", "network", "cloudfront", "cdn", "edge"]),
            icon_item("aws.route53", "Route 53", StencilCategory::Aws, "Network", SymbolIcon::AwsRoute53, &["aws", "network", "route53", "dns"]),
            icon_item("aws.elb", "ELB / ALB", StencilCategory::Aws, "Network", SymbolIcon::AwsElb, &["aws", "network", "elb", "alb", "nlb", "loadbalancer"]),
            icon_item("aws.tgw", "Transit GW", StencilCategory::Aws, "Network", SymbolIcon::AwsTransitGateway, &["aws", "network", "transit", "gateway", "tgw"]),
            icon_item("aws.privatelink", "PrivateLink", StencilCategory::Aws, "Network", SymbolIcon::AwsPrivateLink, &["aws", "network", "privatelink", "endpoint", "vpc"]),
            icon_item("aws.globalacc", "Global Acc", StencilCategory::Aws, "Network", SymbolIcon::AwsGlobalAccelerator, &["aws", "network", "global", "accelerator", "anycast"]),
            icon_item("aws.natgw", "NAT Gateway", StencilCategory::Aws, "Network", SymbolIcon::AwsNatGateway, &["aws", "network", "nat", "gateway"]),
            icon_item("aws.directconnect", "Direct Conn", StencilCategory::Aws, "Network", SymbolIcon::AwsDirectConnect, &["aws", "network", "direct", "connect", "hybrid"]),
            // AWS — Messaging & App Integration
            icon_item("aws.sqs", "SQS", StencilCategory::Aws, "Messaging", SymbolIcon::AwsSqs, &["aws", "messaging", "sqs", "queue"]),
            icon_item("aws.sns", "SNS", StencilCategory::Aws, "Messaging", SymbolIcon::AwsSns, &["aws", "messaging", "sns", "pubsub", "topic"]),
            icon_item("aws.eventbridge", "EventBridge", StencilCategory::Aws, "Messaging", SymbolIcon::AwsEventBridge, &["aws", "messaging", "eventbridge", "events", "bus"]),
            icon_item("aws.stepfn", "Step Fn", StencilCategory::Aws, "Messaging", SymbolIcon::AwsStepFunctions, &["aws", "messaging", "step", "functions", "workflow"]),
            icon_item("aws.appsync", "AppSync", StencilCategory::Aws, "Messaging", SymbolIcon::AwsAppSync, &["aws", "messaging", "appsync", "graphql", "api"]),
            icon_item("aws.ses", "Amazon SES", StencilCategory::Aws, "Messaging", SymbolIcon::AwsSes, &["aws", "messaging", "ses", "email", "smtp"]),
            icon_item("aws.mq", "Amazon MQ", StencilCategory::Aws, "Messaging", SymbolIcon::AwsMq, &["aws", "messaging", "mq", "rabbitmq", "activemq", "broker"]),
            icon_item("aws.amplify", "Amplify", StencilCategory::Aws, "Messaging", SymbolIcon::AwsAmplify, &["aws", "web", "amplify", "frontend", "mobile"]),
            // AWS — Analytics & Streaming
            icon_item("aws.msk", "Amazon MSK", StencilCategory::Aws, "Analytics", SymbolIcon::AwsMsk, &["aws", "analytics", "msk", "kafka", "streaming"]),
            icon_item("aws.flink", "Managed Flink", StencilCategory::Aws, "Analytics", SymbolIcon::AwsFlink, &["aws", "analytics", "flink", "kinesis", "streaming", "apache"]),
            icon_item("aws.kinesis", "Kinesis", StencilCategory::Aws, "Analytics", SymbolIcon::AwsKinesis, &["aws", "analytics", "kinesis", "stream", "data"]),
            icon_item("aws.firehose", "Firehose", StencilCategory::Aws, "Analytics", SymbolIcon::AwsDataFirehose, &["aws", "analytics", "firehose", "kinesis", "etl"]),
            icon_item("aws.athena", "Athena", StencilCategory::Aws, "Analytics", SymbolIcon::AwsAthena, &["aws", "analytics", "athena", "sql", "presto", "trino"]),
            icon_item("aws.glue", "AWS Glue", StencilCategory::Aws, "Analytics", SymbolIcon::AwsGlue, &["aws", "analytics", "glue", "etl", "catalog", "spark"]),
            icon_item("aws.emr", "Amazon EMR", StencilCategory::Aws, "Analytics", SymbolIcon::AwsEmr, &["aws", "analytics", "emr", "spark", "hadoop", "hive"]),
            icon_item("aws.opensearch", "OpenSearch", StencilCategory::Aws, "Analytics", SymbolIcon::AwsOpenSearch, &["aws", "analytics", "opensearch", "elasticsearch", "search", "logs"]),
            icon_item("aws.quicksight", "QuickSight", StencilCategory::Aws, "Analytics", SymbolIcon::AwsQuickSight, &["aws", "analytics", "quicksight", "bi", "dashboard"]),
            icon_item("aws.lakeformation", "Lake Form.", StencilCategory::Aws, "Analytics", SymbolIcon::AwsLakeFormation, &["aws", "analytics", "lakeformation", "datalake"]),
            // AWS — Security, Governance & DevOps
            icon_item("aws.iam", "IAM", StencilCategory::Aws, "Security", SymbolIcon::AwsIam, &["aws", "security", "iam", "role", "identity"]),
            icon_item("aws.cognito", "Cognito", StencilCategory::Aws, "Security", SymbolIcon::AwsCognito, &["aws", "security", "cognito", "auth", "oauth"]),
            icon_item("aws.kms", "KMS", StencilCategory::Aws, "Security", SymbolIcon::AwsKms, &["aws", "security", "kms", "encryption", "key"]),
            icon_item("aws.waf", "AWS WAF", StencilCategory::Aws, "Security", SymbolIcon::AwsWaf, &["aws", "security", "waf", "firewall", "shield"]),
            icon_item("aws.shield", "AWS Shield", StencilCategory::Aws, "Security", SymbolIcon::AwsShield, &["aws", "security", "shield", "ddos"]),
            icon_item("aws.guardduty", "GuardDuty", StencilCategory::Aws, "Security", SymbolIcon::AwsGuardDuty, &["aws", "security", "guardduty", "threat", "siem"]),
            icon_item("aws.secrets", "Secrets Mgr", StencilCategory::Aws, "Security", SymbolIcon::AwsSecretsManager, &["aws", "security", "secrets", "manager", "vault"]),
            icon_item("aws.cloudwatch", "CloudWatch", StencilCategory::Aws, "DevOps", SymbolIcon::AwsCloudWatch, &["aws", "devops", "cloudwatch", "logs", "metrics", "monitor"]),
            icon_item("aws.cloudtrail", "CloudTrail", StencilCategory::Aws, "DevOps", SymbolIcon::AwsCloudTrail, &["aws", "devops", "cloudtrail", "audit", "governance"]),
            icon_item("aws.config", "AWS Config", StencilCategory::Aws, "DevOps", SymbolIcon::AwsConfig, &["aws", "devops", "config", "compliance", "governance"]),
            icon_item("aws.orgs", "Organizations", StencilCategory::Aws, "DevOps", SymbolIcon::AwsOrganizations, &["aws", "devops", "organizations", "accounts"]),
            icon_item("aws.codepipeline", "CodePipeline", StencilCategory::Aws, "DevOps", SymbolIcon::AwsCodePipeline, &["aws", "devops", "codepipeline", "cicd", "deploy"]),
            icon_item("aws.codebuild", "CodeBuild", StencilCategory::Aws, "DevOps", SymbolIcon::AwsCodeBuild, &["aws", "devops", "codebuild", "ci", "build"]),
            icon_item("aws.xray", "AWS X-Ray", StencilCategory::Aws, "DevOps", SymbolIcon::AwsXRay, &["aws", "devops", "xray", "tracing", "apm"]),
            icon_item("aws.ssm", "Systems Mgr", StencilCategory::Aws, "DevOps", SymbolIcon::AwsSsm, &["aws", "devops", "ssm", "systems", "manager", "param"]),
            icon_item("aws.cfn", "CloudForm.", StencilCategory::Aws, "DevOps", SymbolIcon::AwsCloudFormation, &["aws", "devops", "cloudformation", "iac", "cdk"]),
            // AWS — AI & ML
            icon_item("aws.bedrock", "Bedrock", StencilCategory::Aws, "AI & Data", SymbolIcon::AwsBedrock, &["aws", "ai", "ml", "bedrock", "llm", "genai"]),
            icon_item("aws.sagemaker", "SageMaker", StencilCategory::Aws, "AI & Data", SymbolIcon::AwsSageMaker, &["aws", "ai", "ml", "sagemaker", "training"]),
            icon_item("aws.rekognition", "Rekognition", StencilCategory::Aws, "AI & Data", SymbolIcon::AwsRekognition, &["aws", "ai", "ml", "rekognition", "vision"]),
            icon_item("aws.textract", "Textract", StencilCategory::Aws, "AI & Data", SymbolIcon::AwsTextract, &["aws", "ai", "ml", "textract", "ocr"]),
            icon_item("aws.polly", "Amazon Polly", StencilCategory::Aws, "AI & Data", SymbolIcon::AwsPolly, &["aws", "ai", "ml", "polly", "tts", "voice"]),
            icon_item("aws.q", "Amazon Q", StencilCategory::Aws, "AI & Data", SymbolIcon::AwsQ, &["aws", "ai", "ml", "q", "assistant", "genai"]),

            // Azure — Compute
            icon_item("azure.vm", "Virtual Mach", StencilCategory::Azure, "Compute", SymbolIcon::AzureVm, &["azure", "compute", "vm", "virtual", "machine"]),
            icon_item("azure.aks", "Azure AKS", StencilCategory::Azure, "Compute", SymbolIcon::AzureAks, &["azure", "compute", "aks", "kubernetes", "k8s"]),
            icon_item("azure.functions", "Functions", StencilCategory::Azure, "Compute", SymbolIcon::AzureFunctions, &["azure", "compute", "functions", "serverless"]),
            icon_item("azure.appservice", "App Service", StencilCategory::Azure, "Compute", SymbolIcon::AzureAppService, &["azure", "compute", "app", "service", "web"]),
            icon_item("azure.containerapps", "Container App", StencilCategory::Azure, "Compute", SymbolIcon::AzureContainerApps, &["azure", "compute", "container", "apps", "aca"]),
            icon_item("azure.aci", "Container Inst", StencilCategory::Azure, "Compute", SymbolIcon::AzureContainerInstances, &["azure", "compute", "aci", "container", "instances"]),
            icon_item("azure.spring", "Spring Apps", StencilCategory::Azure, "Compute", SymbolIcon::AzureSpringApps, &["azure", "compute", "spring", "java", "apps"]),
            // Azure — Storage
            icon_item("azure.blob", "Blob Storage", StencilCategory::Azure, "Storage", SymbolIcon::AzureBlob, &["azure", "storage", "blob", "object"]),
            icon_item("azure.files", "Azure Files", StencilCategory::Azure, "Storage", SymbolIcon::AzureFiles, &["azure", "storage", "files", "smb", "share"]),
            icon_item("azure.disk", "Managed Disk", StencilCategory::Azure, "Storage", SymbolIcon::AzureDisk, &["azure", "storage", "disk", "block"]),
            icon_item("azure.datalake", "Data Lake", StencilCategory::Azure, "Storage", SymbolIcon::AzureDataLake, &["azure", "storage", "datalake", "adls", "analytics"]),
            // Azure — Database
            icon_item("azure.sql", "Azure SQL", StencilCategory::Azure, "Database", SymbolIcon::AzureSql, &["azure", "database", "sql", "mssql"]),
            icon_item("azure.cosmos", "Cosmos DB", StencilCategory::Azure, "Database", SymbolIcon::AzureCosmosDb, &["azure", "database", "cosmos", "nosql"]),
            icon_item("azure.postgres", "Azure PG", StencilCategory::Azure, "Database", SymbolIcon::AzurePostgres, &["azure", "database", "postgres", "postgresql"]),
            icon_item("azure.redis", "Azure Redis", StencilCategory::Azure, "Database", SymbolIcon::AzureRedis, &["azure", "database", "redis", "cache"]),
            icon_item("azure.synapse", "Synapse", StencilCategory::Azure, "Database", SymbolIcon::AzureSynapse, &["azure", "database", "synapse", "warehouse", "analytics"]),
            // Azure — Networking
            icon_item("azure.vnet", "Virtual Net", StencilCategory::Azure, "Network", SymbolIcon::AzureVnet, &["azure", "network", "vnet", "subnet"]),
            icon_item("azure.frontdoor", "Front Door", StencilCategory::Azure, "Network", SymbolIcon::AzureFrontDoor, &["azure", "network", "frontdoor", "cdn", "edge"]),
            icon_item("azure.appgw", "App Gateway", StencilCategory::Azure, "Network", SymbolIcon::AzureAppGateway, &["azure", "network", "application", "gateway", "waf"]),
            icon_item("azure.lb", "Load Balancer", StencilCategory::Azure, "Network", SymbolIcon::AzureLoadBalancer, &["azure", "network", "load", "balancer", "lb"]),
            icon_item("azure.dns", "Azure DNS", StencilCategory::Azure, "Network", SymbolIcon::AzureDns, &["azure", "network", "dns", "zone"]),
            icon_item("azure.firewall", "Az Firewall", StencilCategory::Azure, "Network", SymbolIcon::AzureFirewall, &["azure", "network", "firewall", "security"]),
            icon_item("azure.apim", "API Mgmt", StencilCategory::Azure, "Network", SymbolIcon::AzureApiManagement, &["azure", "network", "apim", "api", "gateway"]),
            // Azure — Messaging & Integration
            icon_item("azure.servicebus", "Service Bus", StencilCategory::Azure, "Messaging", SymbolIcon::AzureServiceBus, &["azure", "messaging", "servicebus", "queue", "topic"]),
            icon_item("azure.eventhubs", "Event Hubs", StencilCategory::Azure, "Messaging", SymbolIcon::AzureEventHubs, &["azure", "messaging", "eventhubs", "stream", "kafka"]),
            icon_item("azure.eventgrid", "Event Grid", StencilCategory::Azure, "Messaging", SymbolIcon::AzureEventGrid, &["azure", "messaging", "eventgrid", "events"]),
            icon_item("azure.logicapps", "Logic Apps", StencilCategory::Azure, "Messaging", SymbolIcon::AzureLogicApps, &["azure", "messaging", "logic", "workflow"]),
            // Azure — Security & Ops
            icon_item("azure.entraid", "Entra ID", StencilCategory::Azure, "Security", SymbolIcon::AzureEntraId, &["azure", "security", "entra", "ad", "identity", "auth"]),
            icon_item("azure.keyvault", "Key Vault", StencilCategory::Azure, "Security", SymbolIcon::AzureKeyVault, &["azure", "security", "keyvault", "secrets", "kms"]),
            icon_item("azure.monitor", "Az Monitor", StencilCategory::Azure, "Security", SymbolIcon::AzureMonitor, &["azure", "ops", "monitor", "logs", "metrics"]),
            icon_item("azure.devops", "Az DevOps", StencilCategory::Azure, "Security", SymbolIcon::AzureDevOps, &["azure", "ops", "devops", "pipelines", "ci"]),
            // Azure — AI & Data
            icon_item("azure.openai", "Azure OpenAI", StencilCategory::Azure, "AI & Data", SymbolIcon::AzureOpenAi, &["azure", "ai", "openai", "llm", "gpt"]),
            icon_item("azure.aisearch", "AI Search", StencilCategory::Azure, "AI & Data", SymbolIcon::AzureAiSearch, &["azure", "ai", "search", "vector", "cognitive"]),

            // Google Cloud Platform (GCP)
            icon_item("gcp.cloudrun", "Cloud Run", StencilCategory::Gcp, "Compute", SymbolIcon::GcpCloudRun, &["gcp", "google", "cloud", "run", "serverless"]),
            icon_item("gcp.gke", "GKE Cluster", StencilCategory::Gcp, "Compute", SymbolIcon::GcpGke, &["gcp", "google", "gke", "kubernetes", "k8s"]),
            icon_item("gcp.bigquery", "BigQuery", StencilCategory::Gcp, "AI & Data", SymbolIcon::GcpBigQuery, &["gcp", "google", "bigquery", "analytics", "warehouse"]),
            icon_item("gcp.pubsub", "Pub/Sub", StencilCategory::Gcp, "Messaging", SymbolIcon::GcpPubSub, &["gcp", "google", "pubsub", "messaging", "events"]),
            icon_item("gcp.storage", "Cloud Storage", StencilCategory::Gcp, "Storage", SymbolIcon::GcpCloudStorage, &["gcp", "google", "gcs", "bucket", "storage"]),
            icon_item("gcp.sql", "Cloud SQL", StencilCategory::Gcp, "Database", SymbolIcon::GcpCloudSql, &["gcp", "google", "sql", "database", "postgres"]),

            // Infra & DevOps
            icon_item("infra.k8s", "Kubernetes", StencilCategory::InfraDevOps, "", SymbolIcon::Kubernetes, &["kubernetes", "k8s", "cluster", "devops", "container"]),
            icon_item("infra.docker", "Docker", StencilCategory::InfraDevOps, "", SymbolIcon::Docker, &["docker", "container", "devops"]),
            icon_item("infra.postgres", "PostgreSQL", StencilCategory::InfraDevOps, "", SymbolIcon::Postgres, &["postgres", "postgresql", "sql", "db"]),
            icon_item("infra.redis", "Redis Cache", StencilCategory::InfraDevOps, "", SymbolIcon::Redis, &["redis", "cache", "kv", "memory"]),
            icon_item("infra.kafka", "Apache Kafka", StencilCategory::InfraDevOps, "", SymbolIcon::Kafka, &["kafka", "stream", "queue", "events"]),
            icon_item("infra.nginx", "NGINX Proxy", StencilCategory::InfraDevOps, "", SymbolIcon::Nginx, &["nginx", "proxy", "ingress", "web"]),
            icon_item("infra.lb", "Load Balancer", StencilCategory::InfraDevOps, "", SymbolIcon::LoadBalancer, &["lb", "load", "balancer", "router", "network"]),
            icon_item("infra.cdn", "Edge CDN", StencilCategory::InfraDevOps, "", SymbolIcon::Cdn, &["cdn", "edge", "cache", "network"]),

            // Standard System Icons
            icon_item("std.user", "User", StencilCategory::Standard, "", SymbolIcon::User, &["user", "person", "client", "actor"]),
            icon_item("std.users", "Users / Team", StencilCategory::Standard, "", SymbolIcon::Users, &["users", "group", "team", "clients"]),
            icon_item("std.globe", "Internet", StencilCategory::Standard, "", SymbolIcon::Globe, &["globe", "internet", "web", "public"]),
            icon_item("std.lock", "Auth / Lock", StencilCategory::Standard, "", SymbolIcon::Lock, &["lock", "auth", "security", "oauth"]),
            icon_item("std.key", "API Key", StencilCategory::Standard, "", SymbolIcon::Key, &["key", "secret", "kms", "token"]),
            icon_item("std.api", "REST / gRPC", StencilCategory::Standard, "", SymbolIcon::Api, &["api", "rest", "grpc", "endpoint"]),
            icon_item("std.webhook", "Webhook", StencilCategory::Standard, "", SymbolIcon::Webhook, &["webhook", "event", "callback", "hook"]),
            icon_item("std.monitor", "Observability", StencilCategory::Standard, "", SymbolIcon::Monitor, &["monitor", "metrics", "logs", "grafana"]),
            icon_item("std.terminal", "CLI / Worker", StencilCategory::Standard, "", SymbolIcon::Terminal, &["cli", "terminal", "shell", "worker"]),
            icon_item("std.ai", "AI / LLM", StencilCategory::Standard, "", SymbolIcon::AiModel, &["ai", "llm", "ml", "model", "agent"]),

            // Flowchart & ISO
            shape_item("flowchart.process", "Process Card", StencilCategory::Flowchart, ShapeKind::ReactFlowCard, 160.0, 76.0, &["process", "step", "card"]),
            shape_item("flowchart.rounded", "Terminator", StencilCategory::Flowchart, ShapeKind::RoundedRect, 140.0, 60.0, &["start", "end", "terminator"]),
            shape_item("flowchart.decision", "Decision", StencilCategory::Flowchart, ShapeKind::Diamond, 120.0, 100.0, &["decision", "branch", "if"]),
            shape_item("flowchart.database", "Database", StencilCategory::Flowchart, ShapeKind::Cylinder, 120.0, 100.0, &["db", "storage", "sql"]),
            shape_item("flowchart.data_io", "Data I/O", StencilCategory::Flowchart, ShapeKind::Parallelogram, 150.0, 74.0, &["io", "input", "output", "parallelogram"]),
            shape_item("flowchart.document", "Document", StencilCategory::Flowchart, ShapeKind::Document, 150.0, 84.0, &["doc", "file", "report"]),
            shape_item("flowchart.hexagon", "Preparation", StencilCategory::Flowchart, ShapeKind::Hexagon, 150.0, 80.0, &["hex", "init", "setup"]),
            shape_item("flowchart.note", "Sticky Note", StencilCategory::Flowchart, ShapeKind::Note, 150.0, 90.0, &["note", "comment", "annotation"]),
            shape_item("flowchart.callout", "Callout", StencilCategory::Flowchart, ShapeKind::Callout, 150.0, 86.0, &["callout", "speech", "bubble"]),
            shape_item("flowchart.subprocess", "Subprocess", StencilCategory::Flowchart, ShapeKind::Subprocess, 150.0, 64.0, &["flowchart", "subprocess", "predefined"]),
            shape_item("flowchart.manual_input", "Manual Input", StencilCategory::Flowchart, ShapeKind::ManualInput, 140.0, 64.0, &["flowchart", "input", "manual"]),
            shape_item("flowchart.delay", "Delay", StencilCategory::Flowchart, ShapeKind::Delay, 130.0, 60.0, &["flowchart", "delay", "wait"]),
            shape_item("flowchart.connector", "Connector", StencilCategory::Flowchart, ShapeKind::Ellipse, 64.0, 64.0, &["flowchart", "connector", "circle"]),
            shape_item("flowchart.off_page", "Off-Page Ref", StencilCategory::Flowchart, ShapeKind::OffPage, 90.0, 76.0, &["flowchart", "offpage", "reference"]),
            shape_item("flowchart.merge", "Merge", StencilCategory::Flowchart, ShapeKind::Triangle, 100.0, 82.0, &["flowchart", "merge", "triangle"]),

            // Network & Security Shapes
            shape_item("network.server", "Server Rack", StencilCategory::Network, ShapeKind::Server, 130.0, 96.0, &["server", "rack", "host", "node"]),
            shape_item("network.cloud", "Cloud Region", StencilCategory::Network, ShapeKind::Cloud, 160.0, 92.0, &["cloud", "region", "vpc"]),
            shape_item("network.queue", "Message Queue", StencilCategory::Network, ShapeKind::Queue, 165.0, 76.0, &["queue", "kafka", "rabbitmq", "pubsub"]),
            shape_item("network.firewall", "WAF / Firewall", StencilCategory::Network, ShapeKind::Firewall, 150.0, 82.0, &["firewall", "waf", "security"]),
            shape_item("network.shield", "Zero-Trust", StencilCategory::Network, ShapeKind::Shield, 120.0, 100.0, &["auth", "oauth", "shield", "jwt"]),

            // UML
            shape_item("uml.class", "UML Class", StencilCategory::Uml, ShapeKind::ReactFlowCard, 180.0, 110.0, &["uml", "class", "object"]),
            shape_item("uml.interface", "Interface", StencilCategory::Uml, ShapeKind::Interface, 175.0, 90.0, &["uml", "interface", "trait"]),
            shape_item("uml.component", "Component", StencilCategory::Uml, ShapeKind::Component, 170.0, 86.0, &["uml", "component", "module"]),
            shape_item("uml.package", "Package", StencilCategory::Uml, ShapeKind::Package, 180.0, 100.0, &["uml", "package", "crate"]),
            shape_item("uml.actor", "UML Actor", StencilCategory::Uml, ShapeKind::Actor, 80.0, 110.0, &["uml", "actor", "user"]),

            // BPMN
            shape_item("bpmn.task", "BPMN Task", StencilCategory::Bpmn, ShapeKind::RoundedRect, 150.0, 80.0, &["bpmn", "task", "workflow"]),
            shape_item("bpmn.gateway.xor", "XOR Gateway", StencilCategory::Bpmn, ShapeKind::GatewayXor, 90.0, 90.0, &["bpmn", "gateway", "xor"]),
            shape_item("bpmn.gateway.parallel", "AND Gateway", StencilCategory::Bpmn, ShapeKind::GatewayParallel, 90.0, 90.0, &["bpmn", "gateway", "parallel", "and"]),
            shape_item("bpmn.timer", "Timer Event", StencilCategory::Bpmn, ShapeKind::TimerEvent, 84.0, 84.0, &["bpmn", "timer", "cron", "event"]),

            // Wireframes & Containers
            shape_item("wireframe.modal", "Browser", StencilCategory::Wireframes, ShapeKind::BrowserWindow, 220.0, 140.0, &["wireframe", "ui", "window", "browser"]),
            shape_item("wireframe.mobile", "Mobile App", StencilCategory::Wireframes, ShapeKind::MobileScreen, 120.0, 190.0, &["wireframe", "mobile", "ios", "android"]),
            shape_item("wireframe.section", "Section", StencilCategory::Wireframes, ShapeKind::Section, 340.0, 220.0, &["section", "group", "frame", "container"]),
            shape_item("wireframe.image", "Image Node", StencilCategory::Wireframes, ShapeKind::ImageNode, 200.0, 140.0, &["image", "photo", "picture", "media"]),
            shape_item("wireframe.folder_tree", "Folder Tree", StencilCategory::Wireframes, ShapeKind::FolderTree, 210.0, 160.0, &["folder", "tree", "files", "explorer"]),
            shape_item("wireframe.collapsible", "Collapsible", StencilCategory::Wireframes, ShapeKind::Collapsible, 220.0, 130.0, &["collapsible", "accordion", "drawer", "panel"]),
        ];
        Self { items }
    }

    pub fn items(&self) -> &[StencilItem] {
        &self.items
    }

    pub fn search(&self, query: &str) -> Vec<&StencilItem> {
        let q = query.trim().to_ascii_lowercase();
        if q.is_empty() {
            return self.items.iter().collect();
        }
        self.items
            .iter()
            .filter(|item| {
                item.label.to_ascii_lowercase().contains(&q)
                    || item.id.to_ascii_lowercase().contains(&q)
                    || item.subgroup.to_ascii_lowercase().contains(&q)
                    || item
                        .tags
                        .iter()
                        .any(|t| t.to_ascii_lowercase().contains(&q))
            })
            .collect()
    }

    pub fn by_category(&self, category: StencilCategory) -> Vec<&StencilItem> {
        self.items
            .iter()
            .filter(|i| i.category == category)
            .collect()
    }

    pub fn instantiate(
        &self,
        stencil_id: &str,
        node_id: NodeId,
        drop_world: Point,
    ) -> Option<Node> {
        let stencil = self.items.iter().find(|i| i.id == stencil_id)?;
        let bounds = Rect::new(
            drop_world.x,
            drop_world.y,
            stencil.default_width,
            stencil.default_height,
        );
        let mut node = Node::new_react_flow_card(node_id, stencil.label.clone(), bounds);
        node.shape = stencil.shape.clone();
        node.icon = stencil.icon;
        Some(node)
    }
}
