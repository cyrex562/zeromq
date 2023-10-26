use crate::ctx::ZmqContext;
use crate::defines::{ZMQ_RADIO, ZMQ_XPUB_NODROP};
use crate::dist::ZmqDist;
use crate::msg::{MSG_COMMAND, MSG_MORE, ZmqMsg};
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::session_base::ZmqSessionBase;
use crate::socket_base::ZmqSocketBase;
use std::collections::HashMap;
use crate::address::ZmqAddress;
use crate::io_thread::ZmqIoThread;
use std::ffi::c_void;

pub type ZmqSubscriptions<'a> = HashMap<String, &'a mut ZmqPipe<'a>>;
pub type UdpPipes<'a> = Vec<&'a mut ZmqPipe<'a>>;
pub struct ZmqRadio<'a> {
    pub socket_base: ZmqSocketBase<'a>,
    pub _subscriptions: ZmqSubscriptions<'a>,
    pub _udp_pipes: UdpPipes<'a>,
    pub _dist: ZmqDist,
    pub _lossy: bool,
}

impl ZmqRadio {
    pub unsafe fn new(options: &mut ZmqOptions, parent: &mut ZmqContext, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_RADIO;
        Self {
            socket_base: ZmqSocketBase::new(parent, tid_, sid_, false),
            _subscriptions: Default::default(),
            _udp_pipes: vec![],
            _dist: ZmqDist::new(),
            _lossy: false,
        }
    }

    pub unsafe fn xattach_pipe(
        &mut self,
        pipe_: &mut ZmqPipe,
        subscribe_to_all_: bool,
        locally_initiated_: bool,
    ) {
        pipe_.set_nodelay();
        self._dist.attach(pipe_);
        if subscribe_to_all_ {
            self._udp_pipes.push(pipe_);
        } else {
            self.xread_activated(pipe_);
        }
    }

    pub unsafe fn xread_activated(&mut self, pipe_: &mut ZmqPipe) {
        //  There are some subscriptions waiting. Let's process them.
        let mut msg = ZmqMsg::new();
        while pipe_.read(&msg) {
            //  Apply the subscription to the trie
            if (msg.is_join() || msg.is_leave()) {
                let group = (msg.group());

                if (msg.is_join()) {
                    self._subscriptions
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

    pub unsafe fn xwrite_activated(&mut self, pipe_: &mut ZmqPipe) {
        self._dist.activated(pipe_)
    }

    pub unsafe fn xsetsockopt(&mut self, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
        let optval_i32 = i32::from_le_bytes(optval_[0..4].try_into().unwrap());
        if (optvallen_ != 4 || optval_i32 < 0) {
            // errno = EINVAL;
            return -1;
        }
        if (option_ == ZMQ_XPUB_NODROP) {
            self._lossy = optval_i32 == 0;
        } else {
            // errno = EINVAL;
            return -1;
        }
        return 0;
    }

    pub unsafe fn xpipe_terminated(&mut self, pipe_: &mut ZmqPipe) {
        // for (subscriptions_t::iterator it = _subscriptions.begin (), end = _subscriptions.end (); it != end;)
        for it in self._subscriptions.iter_mut() {
            if (it.1 == pipe_) {
                // #if __cplusplus >= 201103L || (defined _MSC_VER && _MSC_VER >= 1700)
                it = self._subscriptions.erase(it);
            // #else
            //             _subscriptions.erase (it++);
            // #endif
            } else {
                // it += 1;
            }
        }

        {
            let end = self._udp_pipes.iter().last();
            // const udp_pipes_t::iterator it =
            //   std::find (_udp_pipes.begin (), end, pipe_);
            let it = self._udp_pipes.iter().find(|&&x| x == pipe_);
            if (it != end) {
                self._udp_pipes.erase(it);
            }
        }

        self._dist.pipe_terminated(pipe_);
    }

    pub unsafe fn xsend(&mut self, msg_: &mut ZmqMsg) -> i32 {
        //  Radio sockets do not allow multipart data (ZMQ_SNDMORE)
        // if (msg_->flags () & msg_t::more)
        if msg_.flag_set(MSG_MORE)
        {
            // errno = EINVAL;
            return -1;
        }

        self._dist.unmatch ();

        // const std::pair<subscriptions_t::iterator, subscriptions_t::iterator>
        //   range = _subscriptions.equal_range (std::string (msg_->group ()));
        let range = self._subscriptions.iter().find(|&&x| x == msg_.group());

        // for (subscriptions_t::iterator it = range.first; it != range.second; ++it)
        //     _dist.match (it->second);
        for it in range {
            self._dist.match_(it.1);
        }

        // for (udp_pipes_t::iterator it = _udp_pipes.begin (),
        //                            end = _udp_pipes.end ();
        //      it != end; ++it)
        //     _dist.match (*it);
        for it in self._udp_pipes.iter_mut() {
            self._dist.match_(it);
        }

        let mut rc = -1;
        if (self._lossy || self._dist.check_hwm ()) {
            if (self._dist.send_to_matching (msg_) == 0) {
                rc = 0; //  Yay, sent successfully
            }
        } else {
            // errno = EAGAIN;
        }

        return rc;
    }

    pub unsafe fn xhas_out(&mut self) -> bool {
        self._dist.has_out()
    }

    pub unsafe fn xrecv(&mut self, msg_: &mut ZmqMsg) -> i32 {
        -1
    }

    pub unsafe fn xhas_in(&mut self) -> bool {
        false
    }
}

pub enum radio_session_state {
    group,
    body,
}

pub struct radio_session_t<'a> {
    pub session_base: ZmqSessionBase<'a>,
    pub _state: radio_session_state,
    pub _pending_msg: ZmqMsg,
}

impl radio_session_t {
    pub unsafe fn new(io_thread_: &mut ZmqIoThread, connect_: bool, socket_: &mut ZmqSocketBase, options_: &ZmqOptions, addr_: ZmqAddress) -> Self {
        Self {
            session_base: ZmqSessionBase::new(io_thread_, connect_, socket_, options_, addr_),
            _state: radio_session_state::group,
            _pending_msg: ZmqMsg::default(),
        }
    }

    pub unsafe fn push_msg(&mut self, msg_: &mut ZmqMsg) -> i32 {
        if msg_.flag_set(MSG_COMMAND) {
            let mut command_data = msg_.data();
            let data_size = msg_.size ();

            let mut group_length = 0usize;
            let mut group = String::new();

            let mut join_leave_msg = ZmqMsg::new();
            let mut rc = 0i32;

            //  Set the msg type to either JOIN or LEAVE
            if data_size >= 5 && command_data.to_string() == "\x04JOIN"
            {
                group_length = (data_size) - 5;
                group = command_data[5..];
                rc = join_leave_msg.init_join ();
            } else if data_size >= 6 && command_data.to_string() == "\x05LEAVE" {
                group_length = (data_size) - 6;
                group = command_data[6..];
                rc = join_leave_msg.init_leave ();
            }
            //  If it is not a JOIN or LEAVE just push the message
            else {
                self.session_base.push_msg(msg_);
                // return session_base_t::push_msg(msg_);
            }

            // errno_assert (rc == 0);

            //  Set the group
            rc = join_leave_msg.set_group (group, group_length);
            // errno_assert (rc == 0);

            //  Close the current command
            rc = msg_.close ();
            // errno_assert (rc == 0);

            //  Push the join or leave command
            *msg_ = join_leave_msg;
            return self.session_base.push_msg (msg_);
        }
        return self.session_base.push_msg (msg_);
    }
    
    pub unsafe fn pull_msg(&mut self, msg_: &mut ZmqMsg) -> i32 {
        if (self._state == radio_session_state::group) {
            let mut rc = self.session_base.pull_msg (&mut self._pending_msg);
            if (rc != 0) {
                return rc;
            }
    
            let  group = self._pending_msg.group ();
            let length = group.len();
    
            //  First frame is the group
            rc = msg_.init_size (length);
            // errno_assert (rc == 0);
            msg_.set_flags (MSG_MORE);
            libc::memcpy (msg_.data () as *mut c_void, group.as_ptr() as *const c_void, length);
    
            //  Next status is the body
            self._state = radio_session_state::body;
            return 0;
        }
        *msg_ = self._pending_msg;
        self._state = radio_session_state::group;
        return 0;
    }
    
    pub unsafe fn reset(&mut self) {
        self.session_base.reset ();
        self._state = radio_session_state::group;
    }
}
