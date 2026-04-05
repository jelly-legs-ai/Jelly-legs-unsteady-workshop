// Analytics & Monitoring Contract - AeTHer Chain
// Network analytics, user behavior tracking, and real-time monitoring

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Event type for tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    // User actions
    UserLogin,
    UserLogout,
    PageView,
    ButtonClick,
    FormSubmit,
    SearchQuery,
    
    // Mining actions
    MiningStart,
    MiningStop,
    RewardClaim,
    DeviceRegister,
    WorkSubmit,
    
    // Staking actions
    StakeCreate,
    StakeUnstake,
    StakeClaim,
    Delegate,
    Undelegate,
    
    // Governance actions
    ProposalView,
    VoteCast,
    ProposalCreate,
    VoteDelegate,
    
    // Agent actions
    AgentRegister,
    AgentKYCSubmit,
    AgentTaskCreate,
    AgentTaskComplete,
    
    // Transaction actions
    TransactionInit,
    TransactionConfirm,
    TransactionFail,
    BridgeInitiate,
    BridgeComplete,
    
    // System events
    EpochStart,
    EpochEnd,
    ValidatorJoin,
    ValidatorLeave,
    SlashingEvent,
    NetworkUpgrade,
}

/// Event severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Tracked event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub event_id: String,
    pub event_type: EventType,
    pub timestamp: u64,
    pub user_address: Option<String>,
    pub device_id: Option<String>,
    pub agent_id: Option<String>,
    pub transaction_hash: Option<String>,
    pub epoch: u64,
    pub metadata: HashMap<String, String>,
    pub severity: EventSeverity,
    pub processed: bool,
    pub indexed: bool,
}

/// Page view analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageViewStats {
    pub url: String,
    pub views: u64,
    pub unique_visitors: u64,
    avg_time_on_page: u64, // milliseconds
    bounce_rate: f64,
    entry_count: u64,
    exit_count: u64,
    by_device: HashMap<String, u64>,
    by_country: HashMap<String, u64>,
    by_referrer: HashMap<String, u64>,
}

/// User session analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub session_id: String,
    pub user_address: String,
    start_time: u64,
    end_time: Option<u64>,
    duration_ms: u64,
    page_views: u64,
    events_triggered: u64,
    transactions_initiated: u64,
    bounce: bool,
    device_type: String,
    country: String,
    referrer: String,
}

/// Funnel conversion tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelStep {
    pub step_name: String,
    pub step_order: u64,
    pub users_entered: u64,
    pub users_completed: u64,
    pub conversion_rate: f64,
    avg_time_to_complete: u64,
    drop_off_count: u64,
    drop_off_rate: f64,
}

/// Cohort analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohortData {
    pub cohort_id: String,
    pub cohort_name: String,
    pub start_date: u64,
    pub user_count: u64,
    pub retention_rates: HashMap<u64, f64>, // epoch -> retention %
    pub active_users: u64,
    pub churned_users: u64,
    pub avg_lifetime_value: f64,
    pub total_rewards_earned: u64,
}

/// Real-time metrics dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeMetrics {
    pub active_users_now: u64,
    pub active_miners_now: u64,
    pub active_validators_now: u64,
    pub transactions_per_minute: u64,
    pub page_views_per_minute: u64,
    pub api_requests_per_minute: u64,
    pub avg_response_time_ms: u64,
    pub error_rate: f64,
    pub uptime_percent: f64,
    pub last_updated: u64,
}

/// Geographic distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoDistribution {
    pub country: String,
    pub user_count: u64,
    pub miner_count: u64,
    pub validator_count: u64,
    pub total_staked: u64,
    pub total_mined: u64,
    pub avg_uptime: f64,
    latitude: f64,
    longitude: f64,
}

/// Device analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAnalytics {
    pub device_type: String,
    pub os: String,
    pub browser: String,
    pub screen_resolution: String,
    pub user_count: u64,
    pub session_count: u64,
    avg_session_duration: u64,
    bounce_rate: f64,
    conversion_rate: f64,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub endpoint: String,
    pub request_count: u64,
    pub avg_response_time_ms: u64,
    pub p50_response_time_ms: u64,
    pub p95_response_time_ms: u64,
    pub p99_response_time_ms: u64,
    pub error_count: u64,
    pub error_rate: f64,
    pub timeout_count: u64,
    pub rate_limit_hits: u64,
}

/// User retention tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionMetrics {
    pub cohort_start: u64,
    pub day_1_retention: f64,
    pub day_7_retention: f64,
    pub day_30_retention: f64,
    pub day_90_retention: f64,
    pub avg_sessions_per_user: f64,
    pub avg_session_duration: u64,
    pub feature_adoption_rate: f64,
}

/// Conversion funnel definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelDefinition {
    pub funnel_id: String,
    pub funnel_name: String,
    pub steps: Vec<String>,
    pub goal_event: EventType,
    pub created_at: u64,
    pub active: bool,
}

/// Analytics & Monitoring contract state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsMonitoring {
    pub events: HashMap<String, Vec<EventRecord>>,
    pub page_views: HashMap<String, PageViewStats>,
    pub sessions: HashMap<String, SessionStats>,
    pub funnels: HashMap<String, FunnelDefinition>,
    pub funnel_results: HashMap<String, Vec<FunnelStep>>,
    pub cohorts: HashMap<String, CohortData>,
    pub geo_distribution: HashMap<String, GeoDistribution>,
    pub device_analytics: HashMap<String, DeviceAnalytics>,
    pub performance_metrics: HashMap<String, PerformanceMetrics>,
    pub retention_metrics: HashMap<String, RetentionMetrics>,
    pub real_time_metrics: RealTimeMetrics,
    pub current_epoch: u64,
    pub total_events_tracked: u64,
    pub total_sessions: u64,
    pub data_retention_epochs: u64,
    pub anonymization_enabled: bool,
    pub gdpr_compliant: bool,
}

impl AnalyticsMonitoring {
    /// Create new analytics monitoring contract
    pub fn new() -> Self {
        let real_time_metrics = RealTimeMetrics {
            active_users_now: 0,
            active_miners_now: 0,
            active_validators_now: 0,
            transactions_per_minute: 0,
            page_views_per_minute: 0,
            api_requests_per_minute: 0,
            avg_response_time_ms: 0,
            error_rate: 0.0,
            uptime_percent: 100.0,
            last_updated: 0,
        };

        AnalyticsMonitoring {
            events: HashMap::new(),
            page_views: HashMap::new(),
            sessions: HashMap::new(),
            funnels: HashMap::new(),
            funnel_results: HashMap::new(),
            cohorts: HashMap::new(),
            geo_distribution: HashMap::new(),
            device_analytics: HashMap::new(),
            performance_metrics: HashMap::new(),
            retention_metrics: HashMap::new(),
            real_time_metrics,
            current_epoch: 0,
            total_events_tracked: 0,
            total_sessions: 0,
            data_retention_epochs: 100,
            anonymization_enabled: true,
            gdpr_compliant: true,
        }
    }

    /// Track an event
    pub fn track_event(
        &mut self,
        event_type: EventType,
        user_address: Option<String>,
        metadata: HashMap<String, String>,
    ) -> String {
        let event_id = format!("evt_{}_{}", self.current_epoch, self.total_events_tracked);
        
        let record = EventRecord {
            event_id: event_id.clone(),
            event_type,
            timestamp: self.get_timestamp(),
            user_address: if self.anonymization_enabled { None } else { user_address },
            device_id: metadata.get("device_id").cloned(),
            agent_id: metadata.get("agent_id").cloned(),
            transaction_hash: metadata.get("tx_hash").cloned(),
            epoch: self.current_epoch,
            metadata,
            severity: self.determine_severity(event_type),
            processed: false,
            indexed: false,
        };

        self.events
            .entry(format!("{}", self.current_epoch))
            .or_insert_with(Vec::new)
            .push(record);

        self.total_events_tracked += 1;
        event_id
    }

    /// Record page view
    pub fn record_page_view(
        &mut self,
        url: String,
        user_agent: String,
        referrer: String,
        country: String,
    ) {
        let stats = self.page_views.entry(url.clone()).or_insert_with(|| PageViewStats {
            url: url.clone(),
            views: 0,
            unique_visitors: 0,
            avg_time_on_page: 0,
            bounce_rate: 0.0,
            entry_count: 0,
            exit_count: 0,
            by_device: HashMap::new(),
            by_country: HashMap::new(),
            by_referrer: HashMap::new(),
        });

        stats.views += 1;
        stats.entry_count += 1;
        *stats.by_device.entry(user_agent).or_insert(0) += 1;
        *stats.by_country.entry(country).or_insert(0) += 1;
        *stats.by_referrer.entry(referrer).or_insert(0) += 1;
    }

    /// Start user session
    pub fn start_session(
        &mut self,
        user_address: String,
        device_type: String,
        country: String,
        referrer: String,
    ) -> String {
        let session_id = format!("sess_{}_{}", self.current_epoch, self.total_sessions);
        
        let session = SessionStats {
            session_id: session_id.clone(),
            user_address,
            start_time: self.get_timestamp(),
            end_time: None,
            duration_ms: 0,
            page_views: 0,
            events_triggered: 0,
            transactions_initiated: 0,
            bounce: true,
            device_type,
            country,
            referrer,
        };

        self.sessions.insert(session_id.clone(), session);
        self.total_sessions += 1;
        session_id
    }

    /// End user session
    pub fn end_session(&mut self, session_id: &str) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.end_time = Some(self.get_timestamp());
            session.duration_ms = session.end_time.unwrap() - session.start_time;
            if session.page_views <= 1 {
                session.bounce = true;
            }
        }
    }

    /// Create conversion funnel
    pub fn create_funnel(
        &mut self,
        name: String,
        steps: Vec<String>,
        goal_event: EventType,
    ) -> String {
        let funnel_id = format!("funnel_{}", name.to_lowercase().replace(' ', "_"));
        
        let funnel = FunnelDefinition {
            funnel_id: funnel_id.clone(),
            funnel_name: name,
            steps,
            goal_event,
            created_at: self.get_timestamp(),
            active: true,
        };

        self.funnels.insert(funnel_id.clone(), funnel);
        funnel_id
    }

    /// Calculate funnel conversion
    pub fn calculate_funnel(&mut self, funnel_id: &str) -> Vec<FunnelStep> {
        if let Some(funnel) = self.funnels.get(funnel_id) {
            let mut steps = Vec::new();
            
            for (i, step_name) in funnel.steps.iter().enumerate() {
                let step = FunnelStep {
                    step_name: step_name.clone(),
                    step_order: i as u64,
                    users_entered: (1000 - i * 100) as u64, // Placeholder
                    users_completed: (800 - i * 80) as u64,  // Placeholder
                    conversion_rate: 0.8,
                    avg_time_to_complete: 30000,
                    drop_off_count: 200,
                    drop_off_rate: 0.2,
                };
                steps.push(step);
            }

            self.funnel_results.insert(funnel_id.to_string(), steps.clone());
            steps
        } else {
            Vec::new()
        }
    }

    /// Update real-time metrics
    pub fn update_real_time_metrics(&mut self) {
        self.real_time_metrics.last_updated = self.get_timestamp();
        self.real_time_metrics.active_users_now = self.sessions.len() as u64;
        // In production, calculate from actual data
        self.real_time_metrics.active_miners_now = 284000;
        self.real_time_metrics.active_validators_now = 12847;
        self.real_time_metrics.transactions_per_minute = 4521;
        self.real_time_metrics.page_views_per_minute = 1200;
        self.real_time_metrics.api_requests_per_minute = 8500;
        self.real_time_metrics.avg_response_time_ms = 45;
        self.real_time_metrics.error_rate = 0.02;
        self.real_time_metrics.uptime_percent = 99.99;
    }

    /// Get geographic distribution
    pub fn get_geo_distribution(&self) -> Vec<GeoDistribution> {
        let mut dist = Vec::new();
        
        dist.push(GeoDistribution {
            country: "United States".to_string(),
            user_count: 85000,
            miner_count: 72000,
            validator_count: 3200,
            total_staked: 15000000,
            total_mined: 4500000,
            avg_uptime: 0.92,
            latitude: 37.0902,
            longitude: -95.7129,
        });

        dist.push(GeoDistribution {
            country: "Germany".to_string(),
            user_count: 42000,
            miner_count: 38000,
            validator_count: 1800,
            total_staked: 8500000,
            total_mined: 2100000,
            avg_uptime: 0.95,
            latitude: 51.1657,
            longitude: 10.4515,
        });

        dist
    }

    /// Determine event severity
    fn determine_severity(&self, event_type: EventType) -> EventSeverity {
        match event_type {
            EventType::TransactionFail | EventType::SlashingEvent => EventSeverity::Error,
            EventType::NetworkUpgrade => EventSeverity::Info,
            EventType::ProposalCreate | EventType::VoteCast => EventSeverity::Info,
            _ => EventSeverity::Info,
        }
    }

    /// Get timestamp (placeholder)
    fn get_timestamp(&self) -> u64 {
        self.current_epoch * 1000
    }

    /// Get events for epoch
    pub fn get_epoch_events(&self, epoch: u64, event_type: Option<EventType>) -> Vec<EventRecord> {
        let events = self.events.get(&format!("{}", epoch));
        match events {
            Some(epoch_events) => {
                if let Some(et) = event_type {
                    epoch_events.iter().filter(|e| e.event_type == et).cloned().collect()
                } else {
                    epoch_events.clone()
                }
            }
            None => Vec::new(),
        }
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &str) -> Option<SessionStats> {
        self.sessions.get(session_id).cloned()
    }

    /// Get page view stats
    pub fn get_page_stats(&self, url: &str) -> Option<PageViewStats> {
        self.page_views.get(url).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_tracking() {
        let mut analytics = AnalyticsMonitoring::new();
        
        let mut metadata = HashMap::new();
        metadata.insert("device_id".to_string(), "device123".to_string());
        metadata.insert("page".to_string(), "/dashboard".to_string());
        
        let event_id = analytics.track_event(
            EventType::PageView,
            Some("user123".to_string()),
            metadata,
        );
        
        assert!(!event_id.is_empty());
        assert_eq!(analytics.total_events_tracked, 1);
    }

    #[test]
    fn test_session_tracking() {
        let mut analytics = AnalyticsMonitoring::new();
        
        let session_id = analytics.start_session(
            "user123".to_string(),
            "Desktop".to_string(),
            "US".to_string(),
            "google".to_string(),
        );
        
        assert!(!session_id.is_empty());
        assert_eq!(analytics.total_sessions, 1);
        
        analytics.end_session(&session_id);
        
        let session = analytics.get_session(&session_id);
        assert!(session.is_some());
        assert!(session.unwrap().duration_ms > 0);
    }

    #[test]
    fn test_funnel_creation() {
        let mut analytics = AnalyticsMonitoring::new();
        
        let funnel_id = analytics.create_funnel(
            "Mining Onboarding".to_string(),
            vec![
                "Landing Page".to_string(),
                "Connect Wallet".to_string(),
                "Register Device".to_string(),
                "Start Mining".to_string(),
            ],
            EventType::MiningStart,
        );
        
        assert!(!funnel_id.is_empty());
        
        let results = analytics.calculate_funnel(&funnel_id);
        assert!(!results.is_empty());
        assert_eq!(results.len(), 4);
    }
}
