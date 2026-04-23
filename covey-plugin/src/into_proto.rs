pub(crate) fn icon(icon: crate::Icon) -> covey_proto::ListItemIcon {
    match icon {
        crate::Icon::Name(name) => covey_proto::ListItemIcon::Name(name),
        crate::Icon::Text(text) => covey_proto::ListItemIcon::Text(text),
    }
}

pub(crate) fn action(action: crate::Action) -> covey_proto::PluginAction {
    action.0
}

pub(crate) fn input(input: crate::Input) -> covey_proto::Input {
    let crate::Input { query, selection } = input;
    covey_proto::Input {
        query,
        selection: selection.to_range(),
    }
}
