use std::ops::Range;

use crate::{Host, Plugin};

pub(crate) fn input(input: covey_proto::Input, plugin: &Plugin) -> crate::Input {
    let covey_proto::Input {
        mut query,
        selection: Range { start, end },
    } = input;

    let prefix = plugin
        .prefix()
        .expect("plugin with no prefix should never be queried");
    let prefix_len = prefix.chars().count();

    query.insert_str(0, prefix);

    crate::Input {
        contents: query,
        selection: (
            start.saturating_add(prefix_len),
            end.saturating_add(prefix_len),
        ),
    }
}

pub(crate) fn list(
    list: covey_proto::List,
    host: &Host,
    plugin: &Plugin,
    request_id: covey_proto::RequestId,
) -> crate::List {
    let covey_proto::List {
        id,
        items,
        style,
        commands: list_commands,
    } = list;

    let style = style.map(self::list_style);
    let list: Vec<_> = items
        .into_iter()
        .map(|item| self::list_item(item, host, plugin))
        .collect();

    crate::List {
        style,
        items: list,
        request_id,
        activation_target: crate::ActivationTarget {
            plugin: plugin.clone(),
            local_target_id: id,
            commands: list_commands,
        },
    }
}

fn list_item(item: covey_proto::ListItem, host: &Host, plugin: &Plugin) -> crate::ListItem {
    let covey_proto::ListItem {
        title,
        description,
        icon,
        id,
        commands: item_commands,
    } = item;

    crate::ListItem {
        activation_target: crate::ActivationTarget {
            plugin: plugin.clone(),
            local_target_id: id,
            commands: item_commands,
        },
        icon: icon.and_then(|icon| self::icon(icon, host)),
        title,
        description,
    }
}

#[expect(clippy::needless_pass_by_value)]
fn list_style(proto: covey_proto::ListStyle) -> crate::ListStyle {
    match proto {
        covey_proto::ListStyle::Rows => crate::ListStyle::Rows,
        covey_proto::ListStyle::Grid => crate::ListStyle::Grid,
        covey_proto::ListStyle::GridWithColumns(columns) => {
            crate::ListStyle::GridWithColumns(columns)
        }
    }
}

fn icon(proto: covey_proto::ListItemIcon, host: &Host) -> Option<crate::ResolvedIcon> {
    // `freedesktop_icons::lookup` can do filesystem reads, which is blocking.
    // Maybe this function should be async. But this is used on the path of turning
    // responses to actions, which is tricky to turn async.
    //
    // Only new icons will need to perform a filesystem lookup. Most icons should be
    // cached, which is a fast lookup and doesn't block.
    match proto {
        covey_proto::ListItemIcon::Name(name) => {
            crate::ResolvedIcon::resolve_icon_name(host, &name).map(crate::ResolvedIcon::File)
        }
        covey_proto::ListItemIcon::Text(text) => Some(crate::ResolvedIcon::Text(text)),
    }
}
