use qlist_api::{export, Plugin};

struct Component;

impl Plugin for Component {
    fn test(name: String) -> Vec<String> {
        vec![name]
    }
}

export!(Component with_types_in qlist_api::bindings);
