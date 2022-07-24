use std::collections::HashMap;

pub type AttrMap = HashMap<String, String>;

pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<Box<Node>>,
}

pub enum NodeType {
    Element(Element),
    Text(Text),
}

#[derive(Debug)]
pub struct Element {
    pub tag_name: String,
    pub attributes: AttrMap,
}

#[derive(Debug)]
pub struct Text {
    pub data: String,
}
