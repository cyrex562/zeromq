use crate::message::ZmqMessage;
use crate::pipe::pipe_t;
use crate::socket_base::ZmqSocketBase;

pub trait ZmqSocketBaseOps {
    // pub get_peer_state_func: Option<GetPeerStateFunc>,
    // pub type GetPeerStateFunc = fn(&mut ZmqSocketBase, routing_id_: &mut [u8], routing_id_size_: usize) -> anyhow::Result<i32>;
    //  Query the state of a specific peer. The default implementation
    //  always returns an ENOTSUP error.
    // virtual int get_peer_state (const routing_id_: *mut c_void,
    //                             routing_id_size_: usize) const;
    fn get_peer_state(&mut self,
                      skt_base: &mut ZmqSocketBase,
                      routing_id_: &mut [u8],
                      routing_id_size: usize) -> anyhow::Result<i32>
    {
        // // LIBZMQ_UNUSED (routing_id_);
        // // LIBZMQ_UNUSED (routing_id_size_);
        //
        // //  Only ROUTER sockets support this
        // // errno = ENOTSUP;
        // // return -1;
        // if self.get_peer_state_func.is_some() {
        //     let f = self.get_peer_state_func.unwrap();
        //     f(self, routing_id_, routing_id_size)
        // }
        // bail!("get peer state not supported")
        unimplemented!()
    }

    // pub xsetsockopt_func: Option<XSetSockOptFunc>,
    // pub type XSetSockOptFunc = fn(&mut ZmqSocketBase, a: i32, b: &mut [u8], c: usize) -> anyhow::Result<()>;
    //  Concrete algorithms for the x- methods are to be defined by
    //  individual socket types.
    // virtual void xattach_pipe (pipe_t *pipe_,
    //                            bool subscribe_to_all_ = false,
    //                            bool locally_initiated_ = false) = 0;
    fn xattach_pipe(&mut self, skt_base: &mut ZmqSocketBase, pipe: &mut pipe_t, subscribe_to_all: bool, locally_initiated: bool) {
        unimplemented!()
    }

    //  The default implementation assumes there are no specific socket
    //  options for the particular socket type. If not so, ZMQ_FINAL this
    //  method.
    // virtual int
    // xsetsockopt (option_: i32, const optval_: *mut c_void, optvallen_: usize);
    fn xsetsockopt(&mut self, skt_base: &mut ZmqSocketBase, a: i32, b: &mut [u8], c: usize) -> anyhow::Result<()>
    {
        unimplemented!()
    }


    // pub xgetsockopt_func: Option<XGetSockOptFunc>,
    // pub type XGetSockOptFunc = fn (&mut ZmqSocketBase, a: i32, b: &mut [u8], c: usize) -> anyhow::Result<()>;
    //  The default implementation assumes there are no specific socket
    //  options for the particular socket type. If not so, ZMQ_FINAL this
    //  method.
    // virtual int xgetsockopt (option_: i32, optval_: *mut c_void, optvallen_: *mut usize);
    fn xgetsockopt(&mut self, skt_base: &mut ZmqSocketBase, a: i32, b: &mut [u8], c: usize) -> anyhow::Result<()>
    {
        unimplemented!()
    }

    // pub xhasout_func: Option<XHasOutFunc>,
    // pub type XHasOutFunc = fn(&mut ZmqSocketBase) -> bool;
    //  The default implementation assumes that send is not supported.
    // virtual bool xhas_out ();
    fn xhas_out(&mut self, skt_base: &mut ZmqSocketBase) -> bool
    {
        unimplemented!()
    }

    // pub xsend_func: Option<XSendFunc>,
    // pub type XSendFunc = fn(&mut ZmqSocketBase, msg: &mut ZmqMessage) -> anyhow::Result<()>;
    // virtual int xsend (ZmqMessage *msg);
    fn xsend(&mut self, skt_base: &mut ZmqSocketBase, msg: &mut ZmqMessage) -> anyhow::Result<()>
    {
        unimplemented!()
    }

    // pub xhasin_func: Option<XHasInFunc>,
    // pub type XHasInFunc = fn(&mut ZmqSocketBase) -> bool;
    //  The default implementation assumes that recv in not supported.
    // virtual bool xhas_in ();
    fn xhas_in(&mut self, skt_base: &mut ZmqSocketBase) -> bool
    {
        // if self.xhasin_func.is_some() {
        //     self.xhasin_func.unwrap()(self)
        // } else {
        //     false
        // }
        unimplemented!()
    }

    // pub xrecv_fn: Option<XRecvFunc>,
    // pub type XRecvFunc = fn(&mut ZmqSocketBase, msg: &mut ZmqMessage) -> anyhow::Result<()>;
    // virtual int xrecv (ZmqMessage *msg);
    fn xrecv(&mut self, skt_base: &mut ZmqSocketBase, msg: &mut ZmqMessage) -> anyhow::Result<()>
    {
        // if self.xrecv_fn.is_some() {
        //     self.xrecv_fn.unwrap()(self,msg)
        // } else {
        //     bail!("not implemented")
        // }
        unimplemented!()
    }

    // pub xjoin_fn: Option<XJoinFunc>,
    // pub type XJoinFunc = fn(&mut ZmqSocketBase, group_: &str) -> anyhow::Result<()>;
    //  the default implementation assumes that joub and leave are not supported.
    // virtual int xjoin (group_: *const c_char);
    fn xjoin(&mut self, skt_base: &mut ZmqSocketBase, group_: &str) -> anyhow::Result<()>
    {
        // if self.xjoin_fn.is_some() {
        //     self.xjoin_fn.unwrap()(self,group_)
        // }
        // bail!("xjoin not supported")
        unimplemented!()
    }

    // pub xleave_fn: Option<XLeaveFunc>,
    // pub type XLeaveFunc = fn(&mut ZmqSocketBase, group_: &str) -> anyhow::Result<()>;
    // virtual int xleave (group_: *const c_char);
    fn xleave(&mut self, skt_base: &mut ZmqSocketBase, group_: &str) -> anyhow::Result<()>
    {
        // if self.xleave_fn.is_some() {
        //     self.xleave_fn.unwrap()(self,group_)
        // }
        // bail!("xleave not supported/implemented")
        unimplemented!()
    }

    // pub xreadactivated_fn: Option<XReadActivatedFunc>,
    // pub type XReadActivatedFunc = fn(&mut ZmqSocketBase, pipe: &mut pipe_t);
    //  i_pipe_events will be forwarded to these functions.
    // virtual void xread_activated (pipe_t *pipe_);
    fn xread_activated(&mut self, skt_base: &mut ZmqSocketBase, pipe: &mut pipe_t)
    {
        // if self.xreadactivated_fn.is_some() {
        //     self.xreadactivated_fn.unwrap()(self,pipe)
        // }
        unimplemented!()
    }

    // pub xwriteactivated_fn: Option<XWriteActivatedFunc>,
    // pub type XWriteActivatedFunc = fn(&mut ZmqSocketBase, pipe: &mut pipe_t);
    // virtual void xwrite_activated (pipe_t *pipe_);
    fn xwrite_activated(&mut self, skt_base: &mut ZmqSocketBase, pipe: &mut pipe_t)
    {
        // if self.xwriteactivated_fn.is_some() {
        //     self.xwriteactivated_fn.unwrap()(self,pipe)
        // }
        unimplemented!()
    }

    // pub xhiccuped_fn: Option<XHiccupedFunc>,
    // pub type XHiccupedFunc = fn(&mut ZmqSocketBase, pipe: & mut pipe_t);
    // virtual void xhiccuped (pipe_t *pipe_);
    fn xhiccuped(&mut self, skt_base: &mut ZmqSocketBase, pipe: &mut pipe_t)
    {
        // if self.xhiccuped_fn.is_some() {
        //     self.xhiccuped_fn.unwrap()(self,pipe)
        // }
        unimplemented!()
    }

    // virtual void xpipe_terminated (pipe_t *pipe_) = 0;
    fn xpipe_terminated(&mut self, skt_base: &mut ZmqSocketBase, pipe: &mut pipe_t) {
        unimplemented!()
    }
}