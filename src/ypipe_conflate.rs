use crate::dbuffer::dbuffer_t;
use crate::ypipe_base::ypipe_base_t;

pub struct ypipe_conflate_t<T: Default + Copy>
{
    pub base: ypipe_base_t<T>,
    pub dbuffer: dbuffer_t<T>,
    pub reader_awake: bool,
}

impl <T: Default + Copy> ypipe_conflate_t<T>
{
    pub fn new() -> Self {
        Self {
            base: ypipe_base_t::new(),
            dbuffer: dbuffer_t::new(),
            reader_awake: false,
        }
    }
}