use crate::domain::host::HostSpec;
use crate::domain::ids::{HostId, WidgetId};
use crate::domain::widget::WidgetSpec;

/// Canonical host and widget identifiers for the default shell.
pub const DEFAULT_BAR_HOST_ID: &str = "bar";
pub const DEFAULT_CALENDAR_HOST_ID: &str = "calendar";
pub const DEFAULT_CLOCK_WIDGET_ID: &str = "clock";
pub const DEFAULT_CALENDAR_WIDGET_ID: &str = "calendar";

/// Declarative specification of an attached widget in a shell composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WidgetAttachmentSpec {
    pub host_id: HostId,
    pub widget_id: WidgetId,
}

/// Declarative specification of a complete shell composition.
///
/// Contains the set of hosts, widgets, their initial visibility,
/// and their canonical attachment relationships.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellSpec {
    pub hosts: Vec<HostSpec>,
    pub widgets: Vec<WidgetSpec>,
    pub attachments: Vec<WidgetAttachmentSpec>,
    pub initially_visible_hosts: Vec<HostId>,
}

impl ShellSpec {
    /// Constructs the standard default Owlenshell composition:
    /// - Top bar host with attached clock widget (initially visible)
    /// - Floating calendar host with attached calendar widget (initially hidden)
    pub fn default_shell() -> Self {
        let bar_host_id = HostId::new(DEFAULT_BAR_HOST_ID);
        let cal_host_id = HostId::new(DEFAULT_CALENDAR_HOST_ID);
        let clock_widget_id = WidgetId::new(DEFAULT_CLOCK_WIDGET_ID);
        let cal_widget_id = WidgetId::new(DEFAULT_CALENDAR_WIDGET_ID);

        let bar_spec = HostSpec::top_bar(bar_host_id.clone(), 32);

        let mut cal_spec = HostSpec::floating(cal_host_id.clone());
        cal_spec.namespace = "owlenshell-calendar".into();

        let clock_spec = WidgetSpec::clock(clock_widget_id.clone());
        let cal_widget_spec = WidgetSpec::calendar(cal_widget_id.clone());

        let attachments = vec![
            WidgetAttachmentSpec {
                host_id: bar_host_id.clone(),
                widget_id: clock_widget_id,
            },
            WidgetAttachmentSpec {
                host_id: cal_host_id.clone(),
                widget_id: cal_widget_id,
            },
        ];

        Self {
            hosts: vec![bar_spec, cal_spec],
            widgets: vec![clock_spec, cal_widget_spec],
            attachments,
            initially_visible_hosts: vec![bar_host_id],
        }
    }

    /// Helper to find a HostSpec by HostId.
    pub fn get_host(&self, id: &HostId) -> Option<&HostSpec> {
        self.hosts.iter().find(|h| &h.id == id)
    }

    /// Helper to find a WidgetSpec by WidgetId.
    pub fn get_widget(&self, id: &WidgetId) -> Option<&WidgetSpec> {
        self.widgets.iter().find(|w| &w.id == id)
    }

    /// Returns whether a host is configured as initially visible.
    pub fn is_initially_visible(&self, id: &HostId) -> bool {
        self.initially_visible_hosts.contains(id)
    }
}
