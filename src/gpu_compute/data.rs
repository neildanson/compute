use std::rc::Rc;

use bytemuck::Pod;

pub enum Data<R: Pod> {
    Slice(Rc<[R]>),
    Single(R),
    Empty(usize),
}

impl<R: Pod> Data<R> {
    pub fn size(&self) -> usize {
        match self {
            Data::Slice(data) => std::mem::size_of::<R>() * data.len(),
            Data::Single(_) => std::mem::size_of::<R>(),
            Data::Empty(size) => *size,
        }
    }
    pub fn bytes(&self) -> Rc<[u8]> {
        match self {
            Data::Slice(data) => Rc::from(bytemuck::cast_slice(data)),
            Data::Single(data) => Rc::from(bytemuck::bytes_of(data)),
            Data::Empty(size) => Rc::from(bytemuck::cast_slice(&vec![0; *size])),
        }
    }
}

impl<T> From<Vec<T>> for Data<T>
where
    T: Pod,
{
    fn from(data: Vec<T>) -> Self {
        Data::Slice(Rc::from(data))
    }
}
