use std::collections::HashMap;

pub struct TypeInstance<Type> {
    pub argument_dimensions: Vec<Type>,
    pub returned_dimension: Type,
}
impl<Type> TypeInstance<Type> {
    pub fn arguments(&self) -> &[Type] {
        &self.argument_dimensions
    }
    pub fn returns(&self) -> &Type {
        &self.returned_dimension
    }
}
pub struct TypeRegister<Type> {
    pub id_to_instances: HashMap<String, Vec<TypeInstance<Type>>>,
}
impl<Type: Default> Default for TypeRegister<Type> {
    fn default() -> Self {
        TypeRegister { id_to_instances: HashMap::new() }
    }
}
impl<Type: Default + Eq> TypeRegister<Type> {
    pub fn new() -> TypeRegister<Type> {
        TypeRegister::default()
    }
    pub fn get_instance(&self, id: &str, look_for: &[Type]) -> Option<&TypeInstance<Type>> {
        if !self.id_to_instances.contains_key(id) {
            return Option::None;
        }
        let instances = self.id_to_instances.get(id).unwrap();
        instances.iter().find(|&instance| instance.arguments() == look_for)
    }
    pub fn add_instance(
        &mut self,
        id: &str,
        argument_dimensions: Vec<Type>,
        returned_dimension: Type,
    ) {
        if self.get_instance(id, &argument_dimensions).is_some() {
            return;
        }
        if !self.id_to_instances.contains_key(id) {
            self.id_to_instances.insert(id.to_string(), Vec::new());
        }
        if let Option::Some(instances) = self.id_to_instances.get_mut(id) {
            let instance = TypeInstance { argument_dimensions, returned_dimension };
            instances.push(instance);
        }
    }
}
