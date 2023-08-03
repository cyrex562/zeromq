use crate::dbuffer::dbuffer_t;
use crate::ypipe_base::ypipe_base_t;

pub struct ypipe_conflate_t<T: Default + Copy>
{
    pub base: ypipe_base_t<T>,
    pub dbuffer: dbuffer_t<T>,
    pub reader_awake: bool,
}