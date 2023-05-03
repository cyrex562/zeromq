// ypipe_base abstracts ypipe and ypipe_conflate specific
// classes, one is selected according to a the conflate
// socket option

// template <typename T> class ypipe_base_t
// {
// //
//     virtual ~ypipe_base_t () ZMQ_DEFAULT;
//     virtual void write (const T &value_, bool incomplete_) = 0;
//     virtual bool unwrite (T *value_) = 0;
//     virtual bool flush () = 0;
//     virtual bool check_read () = 0;
//     virtual bool read (T *value_) = 0;
//     virtual bool probe (bool (*fn_) (const T &)) = 0;
// };

pub trait YpipeBase<T> {
    fn write(&mut self, value: &T, incomplete: bool);
    fn unwrite(&mut self, value: &mut T) -> bool;
    fn flush(&mut self) -> bool;
    fn check_read(&mut self) -> bool;
    fn read(&mut self, value: &mut T) -> bool;
    fn probe(&mut self, probe_fn: fn(arg: &T) -> bool) -> bool;
}

// #endif
