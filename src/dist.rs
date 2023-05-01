/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C+= 1.

    libzmq is free software; you can redistribute it and/or modify it under
    the terms of the GNU Lesser General Public License (LGPL) as published
    by the Free Software Foundation; either version 3 of the License, or
    (at your option) any later version.

    As a special exception, the Contributors give you permission to link
    this library with independent modules to produce an executable,
    regardless of the license terms of these independent modules, and to
    copy and distribute the resulting executable under terms of your choice,
    provided that you also meet, for each linked independent module, the
    terms and conditions of the license of that module. An independent
    module is a module which is not derived from or based on this library.
    If you modify this library, you must extend this exception to your
    version of the library.

    libzmq is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
    FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public
    License for more details.

    You should have received a copy of the GNU Lesser General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

// #include "precompiled.hpp"
// #include "dist.hpp"
// #include "pipe.hpp"
// #include "err.hpp"
// #include "msg.hpp"
// #include "likely.hpp"


//  Class manages a set of outbound pipes. It sends each messages to
//  each of them.
#[derive(Default,Debug,Clone)]
pub struct ZmqDist
{
    //  List of outbound pipes.
    // typedef array_t<ZmqPipe, 2> pipes_t;
    // pipes_t pipes;
    pub pipes: Vec<ZmqPipe>,

    //  Number of all the pipes to send the next message to.
    // pipes_t::size_type matching;
    pub matching: usize,

    //  Number of active pipes. All the active pipes are located at the
    //  beginning of the pipes array. These are the pipes the messages
    //  can be sent to at the moment.
    // pipes_t::size_type active;
    pub active: usize,

    //  Number of pipes eligible for sending messages to. This includes all
    //  the active pipes plus all the pipes that we can in theory send
    //  messages to (the HWM is not yet reached), but sending a message
    //  to them would result in partial message being delivered, ie. message
    //  with initial parts missing.
    // pipes_t::size_type eligible;
    pub eligible: usize,

    //  True if last we are in the middle of a multipart message.
    pub more: bool,

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqDist)
}

impl ZmqDist{
    // public:
    //     ZmqDist ();
    //     ~ZmqDist ();
    // ZmqDist::~ZmqDist ()
    // {
    //     zmq_assert (pipes.empty ());
    // }

    // ZmqDist::ZmqDist () :
    //     matching (0), active (0), eligible (0), more (false)
    pub fn new() -> Self{
        ZmqDist{
            pipes: Vec::new(),
            matching: 0,
            active: 0,
            eligible: 0,
            more: false,
        }
    }

    //     //  Adds the pipe to the distributor object.
    //     void attach (pipe: &mut ZmqPipe);
    pub fn attach (&mut self, pipe: &mut ZmqPipe)
    {
        //  If we are in the middle of sending a message, we'll add new pipe
        //  into the list of eligible pipes. Otherwise we add it to the list
        //  of active pipes.
        if (self.more) {
            self.pipes.push_back (pipe);
            self.pipes.swap (self.eligible, self.pipes.size() - 1);
            self.eligible += 1;
        } else {
            self.pipes.push_back (pipe);
            self.pipes.swap (active, self.pipes.size() - 1);
            active += 1;
            self.eligible += 1;
        }
    }

    //     //  Checks if this pipe is present in the distributor.
    //     bool has_pipe (pipe: &mut ZmqPipe);
    pub fn has_pipe (&mut self, pipe: &mut ZmqPipe) -> bool
    {
        let mut claimed_index = self.pipes.index (pipe);

        // If pipe claims to be outside the available index space it can't be in the distributor.
        if (claimed_index >= self.pipes.size ()) {
            return false;
        }

        return self.pipes[claimed_index] == pipe;
    }

    //     //  Activates pipe that have previously reached high watermark.
    //     void activated (pipe: &mut ZmqPipe);

    //     //  Mark the pipe as matching. Subsequent call to send_to_matching
    //     //  will send message also to this pipe.
    //     void match (pipe: &mut ZmqPipe);
    pub fn mark_matching(&mut self, pipe: &mut ZmqPipe)
    {
        //  If pipe is already matching do nothing.
        if (self.pipes.index (pipe) < self.matching){
            return;}

        //  If the pipe isn't eligible, ignore it.
        if (self.pipes.index (pipe) >= self.eligible){
            return;}

        //  Mark the pipe as matching.
        self.pipes.swap (self.pipes.index (pipe), self.matching);
        self.matching += 1;
    }

    //     //  Marks all pipes that are not matched as matched and vice-versa.
    //     void reverse_match ();
    pub fn reverse_match (&mut self)
    {
        let prev_matching = self.matching;

        // Reset matching to 0
        self.unmatch();

        // Mark all matching pipes as not matching and vice-versa.
        // To do this, push all pipes that are eligible but not
        // matched - i.e. between "matching" and "eligible" -
        // to the beginning of the queue.
        // for (pipes_t::size_type i = prev_matching; i < eligible; += 1i)
        for i in prev_matching .. self.eligible
        {
            self.pipes.swap (i, self.matching += 1);
        }
    }

    //     //  Mark all pipes as non-matching.
    //     void unmatch ();
    pub fn unmatch (&mut self)
    {
        self.matching = 0;
    }

    //     //  Removes the pipe from the distributor object.
    //     void pipe_terminated (pipe: &mut ZmqPipe);
    pub fn pipe_terminated (&mut self, pipe: &mut ZmqPipe)
    {
        //  Remove the pipe from the list; adjust number of matching, active and/or
        //  eligible pipes accordingly.
        if (self.pipes.index (pipe) < self.matching) {
            self.pipes.swap (self.pipes.index (pipe), self.matching - 1);
            self.matching -= 1;
        }
        if (self.pipes.index (pipe) < active) {
            self.pipes.swap (self.pipes.index (pipe), active - 1);
            active -= 1;
        }
        if (self.pipes.index (pipe) < self.eligible) {
            self.pipes.swap (self.pipes.index (pipe), self.eligible - 1);
            self.eligible -= 1;
        }

        self.pipes.erase (pipe);
    }

    //     //  Send the message to the matching outbound pipes.
    //     int send_to_matching (msg: &mut ZmqMessage);
    pub fn activated (&mut self, pipe: &mut ZmqPipe)
    {
        //  Move the pipe from passive to eligible state.
        if (self.eligible < self.pipes.size ()) {
            self.pipes.swap (self.pipes.index (pipe), self.eligible);
            self.eligible += 1;
        }

        //  If there's no message being sent at the moment, move it to
        //  the active state.
        if (!self.more && active < self.pipes.size ()) {
            self.pipes.swap (self.eligible - 1, self.active);
            self.active += 1;
        }
    }

    //     //  Send the message to all the outbound pipes.
    //     int send_to_all (msg: &mut ZmqMessage);
    pub fn send_to_all (&mut self, msg: &mut ZmqMessage) -> i32
    {
        self.matching = self.active;
        return self.send_to_matching (msg);
    }

    //     static bool has_out ();
    pub fn send_to_matching (msg: &mut ZmqMessage) -> i32
    {
        //  Is this end of a multipart message?
        let msg_more = (msg.flags () & ZMQ_MSG_MORE) != 0;

        //  Push the message to matching pipes.
        self.distribute (msg);

        //  If multipart message is fully sent, activate all the eligible pipes.
        if (!msg_more){
            self.active = self.eligible;}

        self.more = msg_more;

        return 0;
    }

    //     // check HWM of all pipes matching
    //     bool check_hwm ();

    //   // private:
    //     //  Write the message to the pipe. Make the pipe inactive if writing
    //     //  fails. In such a case false is returned.
    //     bool write (pipe: &mut ZmqPipe, msg: &mut ZmqMessage);

    //     //  Put the message to all active pipes.
    //     void distribute (msg: &mut ZmqMessage);
    pub fn distribute (&mut self, msg: &mut ZmqMessage)
    {
        //  If there are no matching pipes available, simply drop the message.
        if (self.matching == 0) {
            let mut rc = msg.close ();
            // errno_assert (rc == 0);
            rc = msg.init ();
            // errno_assert (rc == 0);
            return;
        }

        if (msg.is_vsm ()) {
            // for (pipes_t::size_type i = 0; i < matching;)
            for i in 0 .. self.matching
            {
                if (!self.write (self.pipes[i], msg)) {
                    //  Use same index again because entry will have been removed.
                } else {
                    i += 1;
                }
            }
            let mut rc = msg.init();
            // errno_assert (rc == 0);
            return;
        }

        //  Add matching-1 references to the message. We already hold one reference,
        //  that's why -1.
        msg.add_refs (self.matching - 1);

        //  Push copy of the message to each matching pipe.
        let mut failed = 0;
        // for (pipes_t::size_type i = 0; i < matching;)
        for i in 0 .. self.matching
        {
            if (!self.write (self.pipes[i], msg)) {
                failed += 1;
                //  Use same index again because entry will have been removed.
            } else {
                i += 1;
            }
        }
        if (unlikely (failed)){
            msg.rm_refs (failed);}

        //  Detach the original message from the data buffer. Note that we don't
        //  close the message. That's because we've already used all the references.
        let rc: i32 = msg.init ();
        // errno_assert (rc == 0);
    }


    pub fn has_out (&self) -> bool
    {
        return true;
    }

    pub fn write (pipe: &mut ZmqPipe, msg: &mut ZmqMessage) -> bool
    {
        if (!pipe.write (msg)) {
            self.pipes.swap (self.pipes.index (pipe), self.matching - 1);
            self.matching -= 1;
            self.pipes.swap (self.pipes.index (pipe), self.active - 1);
            self.active -= 1;
            self.pipes.swap (self.active, self.eligible - 1);
            self.eligible -= 1;
            return false;
        }
        if (!(msg.flags () & ZMQ_MSG_MORE)){
            pipe.flush ();}
        return true;
    }

    pub fn check_hwm (&self) -> bool
    {
        // for (pipes_t::size_type i = 0; i < matching; += 1i)
        for i in 0 .. self.matching
        {
            if (!self.pipes[i].check_hwm ()){
                return false;}}

        return true;
    }
}