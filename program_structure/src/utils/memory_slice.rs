use num_bigint_dig::BigInt;
use std::fmt::{Display, Formatter};

pub enum TypeInvalidAccess{
    MissingInputs(String),
    MissingInputTags(String),
    NoInitializedComponent,
    NoInitializedSignal
}

pub enum TypeAssignmentError{
    MultipleAssignments,
    AssignmentOutput
}

pub enum MemoryError {
    OutOfBoundsError,
    AssignmentError(TypeAssignmentError),
    InvalidAccess(TypeInvalidAccess),
    UnknownSizeDimension,
    MismatchedDimensions(usize, usize),
    MismatchedDimensionsWeak(usize, usize),
    AssignmentMissingTags(String),
    AssignmentTagAfterInit,
    AssignmentTagTwice,
    AssignmentTagInputTwice(String),
    AssignmentTagInput,
    TagValueNotInitializedAccess,
    MissingInputs(String)
}
pub type SliceCapacity = usize;
pub type SimpleSlice = MemorySlice<BigInt>;
/*
    Represents the value stored in a element of a circom program.
    The attribute route stores the dimensions of the slice, used to navigate through them.
    The length of values is equal to multiplying all the values in route.
*/
#[derive(Eq, PartialEq)]
pub struct MemorySlice<C> {
    route: Vec<SliceCapacity>,
    values: Vec<C>,
    number_inserts: usize,
}

impl<C: Clone> Clone for MemorySlice<C> {
    fn clone(&self) -> Self {
        MemorySlice { route: self.route.clone(), values: self.values.clone(), number_inserts: self.number_inserts}
    }
}

impl<C: Default + Clone + Display + Eq> Display for MemorySlice<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.values.is_empty() {
            f.write_str("[]")
        } else if self.values.len() == 1 {
            f.write_str(&format!("{}", self.values[0]))
        } else {
            let mut msg = format!("[{}", self.values[0]);
            for i in 1..self.values.len() {
                msg.push_str(&format!(",{}", self.values[i]));
            }
            msg.push_str("]");
            f.write_str(&msg)
        }
    }
}

impl<C: Clone> MemorySlice<C> {
    // Raw manipulations of the slice
    fn get_initial_cell(
        memory_slice: &MemorySlice<C>,
        access: &[SliceCapacity],
    ) -> Result<SliceCapacity, MemoryError> {
        
        if access.len() > memory_slice.route.len() {
            return Result::Err(MemoryError::OutOfBoundsError);
        }

        let mut cell = 0;
        let mut cell_jump = memory_slice.values.len();
        let mut i: SliceCapacity = 0;
        while i < access.len() {
            if access[i] >= memory_slice.route[i] {
                return Result::Err(MemoryError::OutOfBoundsError);
            }
            cell_jump /= memory_slice.route[i];
            cell += cell_jump * access[i];
            i += 1;
        }
        Result::Ok(cell)
    }
    pub fn check_correct_dims(
        memory_slice: &MemorySlice<C>,
        access: &[SliceCapacity],
        new_values: &MemorySlice<C>,
        is_strict: bool,
    ) -> Result<(), MemoryError> {

        if access.len() + new_values.route.len() > memory_slice.route.len() {
            return Result::Err(MemoryError::OutOfBoundsError);
        }

        let mut i: SliceCapacity = 0;
        
        while i < access.len() {
            if access[i] >= memory_slice.route[i] {
                return Result::Err(MemoryError::OutOfBoundsError);
            }
            i += 1;
        }

        let initial_index_new: SliceCapacity = i;
        i = 0;

        while i < new_values.route.len() {
            if new_values.route[i] < memory_slice.route[initial_index_new + i] {
                if is_strict{ // case signals: we do not allow 
                    return Result::Err(MemoryError::MismatchedDimensions(new_values.route[i], memory_slice.route[initial_index_new + i]));
                } else{ // case variables: we allow the assignment of smaller arrays
                    return Result::Err(MemoryError::MismatchedDimensionsWeak(new_values.route[i], memory_slice.route[initial_index_new + i]));
                }
            }
            if new_values.route[i] > memory_slice.route[initial_index_new + i] {
                return Result::Err(MemoryError::MismatchedDimensions(new_values.route[i], memory_slice.route[initial_index_new + i]));
            }
            i += 1;
        }
        Result::Ok(())
    }

    // Returns the new route and the total number of cells
    // that a slice with such route will have
    fn generate_new_route_from_access(
        memory_slice: &MemorySlice<C>,
        access: &[SliceCapacity],
    ) -> Result<(Vec<SliceCapacity>, SliceCapacity), MemoryError> {
        if access.len() > memory_slice.route.len() {
            return Result::Err(MemoryError::OutOfBoundsError);
        }

        let mut size = Vec::new();
        let mut number_of_cells = 1;
        for i in access.len()..memory_slice.route.len() {
            number_of_cells *= memory_slice.route[i];
            size.push(memory_slice.route[i]);
        }
        Result::Ok((size, number_of_cells))
    }

    fn generate_slice_from_access(
        memory_slice: &MemorySlice<C>,
        access: &[SliceCapacity],
    ) -> Result<MemorySlice<C>, MemoryError> {
        if access.is_empty() {
            return Result::Ok(memory_slice.clone());
        }

        let (size, number_of_cells) =
            MemorySlice::generate_new_route_from_access(memory_slice, access)?;
        let mut values = Vec::with_capacity(number_of_cells);
        let initial_cell = MemorySlice::get_initial_cell(memory_slice, access)?;
        let mut offset = 0;
        while offset < number_of_cells {
            let new_value = memory_slice.values[initial_cell + offset].clone();
            values.push(new_value);
            offset += 1;
        }

        Result::Ok(MemorySlice { route: size, values, number_inserts: 0 })
    }

    // User operations
    pub fn new(initial_value: &C) -> MemorySlice<C> {
        MemorySlice::new_with_route(&[], initial_value)
    }
    pub fn new_array(route: Vec<SliceCapacity>, values: Vec<C>) -> MemorySlice<C> {
        MemorySlice { route, values, number_inserts: 0 }
    }
    pub fn new_with_route(route: &[SliceCapacity], initial_value: &C) -> MemorySlice<C> {
        let mut length = 1;
        for i in route {
            length *= *i;
        }

        let mut values = Vec::with_capacity(length);
        for _i in 0..length {
            values.push(initial_value.clone());
        }

        MemorySlice { route: route.to_vec(), values, number_inserts: 0 }
    }
    pub fn insert_values(
        memory_slice: &mut MemorySlice<C>,
        access: &[SliceCapacity],
        new_values: &MemorySlice<C>,
        is_strict:bool,
    ) -> Result<(), MemoryError> {
        match MemorySlice::check_correct_dims(memory_slice, access, new_values, is_strict){
            Result::Ok(_) => {
                let mut cell = MemorySlice::get_initial_cell(memory_slice, access)?;

                // if MemorySlice::get_number_of_cells(new_values)
                //     > (MemorySlice::get_number_of_cells(memory_slice) - cell)
                // {
                //     return Result::Err(MemoryError::OutOfBoundsError);
                
                memory_slice.number_inserts += MemorySlice::get_number_of_cells(new_values); 
                for value in new_values.values.iter() {
                    memory_slice.values[cell] = value.clone();
                    cell += 1;
                }
                Result::Ok(())
            },
            Result::Err(MemoryError::MismatchedDimensionsWeak(dim_1, dim_2)) => {
                let mut cell = MemorySlice::get_initial_cell(memory_slice, access)?;
                // if MemorySlice::get_number_of_cells(new_values)
                //     > (MemorySlice::get_number_of_cells(memory_slice) - cell)
                // {
                //     return Result::Err(MemoryError::OutOfBoundsError);
                // }
                for value in new_values.values.iter() {
                    memory_slice.values[cell] = value.clone();
                    cell += 1;
                }
                Result::Err(MemoryError::MismatchedDimensionsWeak(dim_1, dim_2))
            },
            Result::Err(error) => return Err(error),
        }
    }

    pub fn insert_value_by_index(
        memory_slice: &mut MemorySlice<C>,
        index: usize,
        new_value: C,
    )-> Result<(), MemoryError> {
        if index > MemorySlice::get_number_of_cells(memory_slice) {
            return Result::Err(MemoryError::OutOfBoundsError);
        }
        memory_slice.number_inserts += 1;
        memory_slice.values[index] = new_value;
        return Result::Ok(());
    }

    pub fn get_access_index(
        memory_slice: &MemorySlice<C>,
        index: usize,
    ) -> Result<Vec<SliceCapacity>, MemoryError>{
        let mut number_cells = MemorySlice::get_number_of_cells(memory_slice);
        if index > number_cells {
            return Result::Err(MemoryError::OutOfBoundsError);
        }
        else{
            let mut access = vec![];
            let mut index_aux = index;
            for pos in &memory_slice.route{
                number_cells = number_cells/pos;
                access.push(index_aux / number_cells);
                index_aux = index_aux % number_cells;
            }
            return Result::Ok(access);
        }
    }

    pub fn access_values(
        memory_slice: &MemorySlice<C>,
        access: &[SliceCapacity],
    ) -> Result<MemorySlice<C>, MemoryError> {
        MemorySlice::generate_slice_from_access(memory_slice, access)
    }
    pub fn access_value_by_index(
        memory_slice: &MemorySlice<C>,
        index: usize,
    )-> Result<C, MemoryError> {
        if index > MemorySlice::get_number_of_cells(memory_slice) {
            return Result::Err(MemoryError::OutOfBoundsError);
        }
        return Result::Ok(memory_slice.values[index].clone());
    }

    pub fn get_reference_to_single_value<'a>(
        memory_slice: &'a MemorySlice<C>,
        access: &[SliceCapacity],
    ) -> Result<&'a C, MemoryError> {
        assert_eq!(memory_slice.route.len(), access.len());
        let cell = MemorySlice::get_initial_cell(memory_slice, access)?;
        Result::Ok(&memory_slice.values[cell])
    }
    pub fn get_reference_to_single_value_by_index<'a>(
        memory_slice: &'a MemorySlice<C>,
        index: usize,
    ) -> Result<&'a C, MemoryError> {
        if index > MemorySlice::get_number_of_cells(memory_slice) {
            return Result::Err(MemoryError::OutOfBoundsError);
        }
        Result::Ok(&memory_slice.values[index])
    }
    pub fn get_reference_to_single_value_by_index_or_break<'a>(
        memory_slice: &'a MemorySlice<C>,
        index: usize,
    ) -> &'a C {
        if index > MemorySlice::get_number_of_cells(memory_slice) {
            unreachable!("The index is too big for the slice");
        }
        &memory_slice.values[index]
    }
    pub fn get_mut_reference_to_single_value<'a>(
        memory_slice: &'a mut MemorySlice<C>,
        access: &[SliceCapacity],
    ) -> Result<&'a mut C, MemoryError> {
        assert_eq!(memory_slice.route.len(), access.len());
        let cell = MemorySlice::get_initial_cell(memory_slice, access)?;
        Result::Ok(&mut memory_slice.values[cell])
    }
    pub fn get_number_of_cells(memory_slice: &MemorySlice<C>) -> SliceCapacity {
        memory_slice.values.len()
    }
    pub fn get_number_of_inserts(memory_slice: &MemorySlice<C>) -> SliceCapacity {
        memory_slice.number_inserts
    }
    pub fn route(&self) -> &[SliceCapacity] {
        &self.route
    }
    pub fn is_single(&self) -> bool {
        self.route.is_empty()
    }
    /*
        !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
        !   Calling this function with a MemorySlice  !
        !   that has more than one cell will cause    !
        !   the compiler to panic.  Use carefully     !
        !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    */
    pub fn unwrap_to_single(memory_slice: MemorySlice<C>) -> C {
        assert!(memory_slice.is_single());
        let mut memory_slice = memory_slice;
        memory_slice.values.pop().unwrap()
    }
    pub fn destruct(self) -> (Vec<SliceCapacity>, Vec<C>) {
        (self.route, self.values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type U32Slice = MemorySlice<u32>;

    #[test]
    fn memory_slice_vector_initialization() {
        let route = vec![3, 4];
        let slice = U32Slice::new_with_route(&route, &0);
        assert_eq!(U32Slice::get_number_of_cells(&slice), 12);
        for (dim_0, dim_1) in slice.route.iter().zip(&route) {
            assert_eq!(*dim_0, *dim_1);
        }
        for f in 0..3 {
            for c in 0..4 {
                let result = U32Slice::get_reference_to_single_value(&slice, &[f, c]);
                if let Result::Ok(v) = result {
                    assert_eq!(*v, 0);
                } else {
                    assert!(false);
                }
            }
        }
    }
    #[test]
    fn memory_slice_single_initialization() {
        let slice = U32Slice::new(&4);
        assert_eq!(U32Slice::get_number_of_cells(&slice), 1);
        let memory_response = U32Slice::get_reference_to_single_value(&slice, &[]);
        if let Result::Ok(val) = memory_response {
            assert_eq!(*val, 4);
        } else {
            assert!(false);
        }
    }
    #[test]
    fn memory_slice_multiple_insertion() {
        let route = vec![3, 4];
        let mut slice = U32Slice::new_with_route(&route, &0);
        let new_row = U32Slice::new_with_route(&[4], &4);

        let res = U32Slice::insert_values(&mut slice, &[2], &new_row);
        if let Result::Ok(_) = res {
            for c in 0..4 {
                let memory_result = U32Slice::get_reference_to_single_value(&slice, &[2, c]);
                if let Result::Ok(val) = memory_result {
                    assert_eq!(*val, 4);
                } else {
                    assert!(false);
                }
            }
        } else {
            assert!(false);
        }
    }
}
