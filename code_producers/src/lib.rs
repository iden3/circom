#[allow(dead_code)]
pub mod c_elements;
#[allow(dead_code)]
pub mod wasm_elements;

pub mod components;


#[derive(Default, Clone)]
pub struct FieldData{
    pub dimensions: Vec<usize>,
    pub size: usize,
    pub offset: usize,
    pub bus_id: Option<usize>
}

pub type FieldMap = Vec<Vec<FieldData>>;
