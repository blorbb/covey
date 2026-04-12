use std::ops::Range;

use crate::Plugin;

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
        .map(|item| self::list_item(item, plugin))
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

fn list_item(item: covey_proto::ListItem, plugin: &Plugin) -> crate::ListItem {
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
        icon: icon.map(self::icon),
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

fn icon(proto: covey_proto::ListItemIcon) -> crate::Icon {
    crate::Icon(proto)
}
