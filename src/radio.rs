use crate::ctx::ZmqContext;
use crate::defines::{MSG_MORE, ZMQ_RADIO, ZMQ_XPUB_NODROP};
use crate::dist::ZmqDist;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocket;
use std::collections::HashMap;

pub type ZmqSubscriptions<'a> = HashMap<String, &'a mut ZmqPipe<'a>>;
pub type UdpPipes<'a> = Vec<&'a mut ZmqPipe<'a>>;
pub struct ZmqRadio<'a> {
    pub socket_base: ZmqSocket<'a>,
    pub _subscriptions: ZmqSubscriptions<'a>,
    pub _udp_pipes: UdpPipes<'a>,
    pub _dist: ZmqDist,
    pub _lossy: bool,
}

impl ZmqRadio {
    pub unsafe fn new(
        options: &mut ZmqOptions,
        parent: &mut ZmqContext,
        tid_: u32,
        sid_: i32,
    ) -> Self {
        options.type_ = ZMQ_RADIO;
        Self {
            socket_base: ZmqSocket::new(parent, tid_, sid_, false),
            _subscriptions: Default::default(),
            _udp_pipes: vec![],
            _dist: ZmqDist::new(),
            _lossy: false,
        }
    }
}

pub fn radio_xsetsockopt(
    socket: &mut ZmqSocket,
    option_: i32,
    optval_: &[u8],
    optvallen_: usize,
) -> i32 {
    unimplemented!()
}

pub unsafe fn radio_xattach_pipe(
    socket: &mut ZmqSocket,
    pipe_: &mut ZmqPipe,
    subscribe_to_all_: bool,
    locally_initiated_: bool,
) {
    pipe_.set_nodelay();
    socket.dist.attach(pipe_);
    if subscribe_to_all_ {
        socket.udp_pipes.push(pipe_);
    } else {
        socket.xread_activated(pipe_);
    }
}

pub unsafe fn radio_xread_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    //  There are some subscriptions waiting. Let's process them.
    let mut msg = ZmqMsg::new();
    while pipe_.read(&msg) {
        //  Apply the subscription to the trie
        if (msg.is_join() || msg.is_leave()) {
            let group = (msg.group());

            if (msg.is_join()) {
                socket
                    .subscriptions
                    .ZMQ_MAP_INSERT_OR_EMPLACE((group), pipe_);
            } else {
                // std::pair<subscriptions_t::iterator, subscriptions_t::iterator>
                //   range = _subscriptions.equal_range (group);
                //
                // for (subscriptions_t::iterator it = range.first;
                //      it != range.second; ++it) {
                //     if (it->second == pipe_) {
                //         _subscriptions.erase (it);
                //         break;
                //     }
                // }
            }
        }
        msg.close();
    }
}

pub unsafe fn radio_xwrite_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket.dist.activated(pipe_)
}

pub unsafe fn xsetsockopt(
    socket: &mut ZmqSocket,
    option_: i32,
    optval_: &[u8],
    optvallen_: usize,
) -> i32 {
    let optval_i32 = i32::from_le_bytes(optval_[0..4].try_into().unwrap());
    if (optvallen_ != 4 || optval_i32 < 0) {
        // errno = EINVAL;
        return -1;
    }
    if (option_ == ZMQ_XPUB_NODROP) {
        socket.lossy = optval_i32 == 0;
    } else {
        // errno = EINVAL;
        return -1;
    }
    return 0;
}

pub unsafe fn radio_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    // for (subscriptions_t::iterator it = _subscriptions.begin (), end = _subscriptions.end (); it != end;)
    for it in socket.subscriptions.iter_mut() {
        if (it.1 == pipe_) {
            // #if __cplusplus >= 201103L || (defined _MSC_VER && _MSC_VER >= 1700)
            it = socket.subscriptions.erase(it);
        // #else
        //             _subscriptions.erase (it++);
        // #endif
        } else {
            // it += 1;
        }
    }

    {
        let end = socket.udp_pipes.iter().last();
        // const udp_pipes_t::iterator it =
        //   std::find (_udp_pipes.begin (), end, pipe_);
        let it = socket.udp_pipes.iter().find(|&&x| x == pipe_);
        if (it != end) {
            socket.udp_pipes.erase(it);
        }
    }

    socket.dist.pipe_terminated(pipe_);
}

pub fn radio_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    //  Radio sockets do not allow multipart data (ZMQ_SNDMORE)
    // if (msg_->flags () & msg_t::more)
    if msg_.flag_set(MSG_MORE) {
        // errno = EINVAL;
        return -1;
    }

    socket.dist.unmatch();

    // const std::pair<subscriptions_t::iterator, subscriptions_t::iterator>
    //   range = _subscriptions.equal_range (std::string (msg_->group ()));
    let range = socket.subscriptions.iter().find(|&&x| x == msg_.group());

    // for (subscriptions_t::iterator it = range.first; it != range.second; ++it)
    //     _dist.match (it->second);
    for it in range {
        socket.dist.match_(it.1);
    }

    // for (udp_pipes_t::iterator it = _udp_pipes.begin (),
    //                            end = _udp_pipes.end ();
    //      it != end; ++it)
    //     _dist.match (*it);
    for it in socket.udp_pipes.iter_mut() {
        socket.dist.match_(it);
    }

    let mut rc = -1;
    if (socket.lossy || self._dist.check_hwm()) {
        if (socket.dist.send_to_matching(msg_) == 0) {
            rc = 0; //  Yay, sent successfully
        }
    } else {
        // errno = EAGAIN;
    }

    return rc;
}

pub fn radio_xhas_out(socket: &mut ZmqSocket) -> bool {
    socket.dist.has_out()
}

pub unsafe fn radio_xrecv(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    -1
}

pub fn radio_xhas_in(socket: &mut ZmqSocket) -> bool {
    false
}

pub fn radio_xgetsockopt(socket: &mut ZmqSocket, option: u32) -> Result<[u8], ZmqError> {
    unimplemented!();
}

pub fn radio_xjoin(socket: &mut ZmqSocket, group: &str) -> i32 {
    unimplemented!();
}
