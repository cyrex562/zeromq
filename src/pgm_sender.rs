/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C++.

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

// #if defined ZMQ_HAVE_OPENPGM

// #include <stdlib.h>

// #include "io_thread.hpp"
// #include "pgm_sender.hpp"
// #include "session_base.hpp"
// #include "err.hpp"
// #include "wire.hpp"
// #include "stdint.hpp"
// #include "macros.hpp"
pub struct pgm_sender_t ZMQ_FINAL : public io_object_t, public i_engine
{
// public:
    pgm_sender_t (io_thread_t *parent_, const ZmqOptions &options_);
    ~pgm_sender_t ();

    int init (udp_encapsulation_: bool, network_: &str);

    //  i_engine interface implementation.
    bool has_handshake_stage () { return false; };
    void plug (io_thread_t *io_thread_, session_base_t *session_);
    void terminate ();
    bool restart_input ();
    void restart_output ();
    void zap_msg_available () {}
    const EndpointUriPair &get_endpoint () const;

    //  i_poll_events interface implementation.
    void in_event ();
    void out_event ();
    void timer_event (token: i32);

  // private:
    //  Unplug the engine from the session.
    void unplug ();

    //  TX and RX timeout timer ID's.
    enum
    {
        tx_timer_id = 0xa0,
        rx_timer_id = 0xa1
    };

    const EndpointUriPair _empty_endpoint;

    //  Timers are running.
    has_tx_timer: bool
    has_rx_timer: bool

    session_base_t *session;

    //  Message encoder.
    v1_encoder_t encoder;

    ZmqMessage msg;

    //  Keeps track of message boundaries.
    more_flag: bool

    //  PGM socket.
    pgm_socket_t pgm_socket;

    //  Socket options.
    ZmqOptions options;

    //  Poll handle associated with PGM socket.
    handle_t handle;
    handle_t uplink_handle;
    handle_t rdata_notify_handle;
    handle_t pending_notify_handle;

    //  Output buffer from pgm_socket.
    unsigned char *out_buffer;

    //  Output buffer size.
    out_buffer_size: usize;

    //  Number of bytes in the buffer to be written to the socket.
    //  If zero, there are no data to be sent.
    write_size: usize;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (pgm_sender_t)
};

pgm_sender_t::pgm_sender_t (io_thread_t *parent_,
                                 const ZmqOptions &options_) :
    io_object_t (parent_),
    has_tx_timer (false),
    has_rx_timer (false),
    session (null_mut()),
    encoder (0),
    more_flag (false),
    pgm_socket (false, options_),
    options (options_),
    handle (static_cast<handle_t> (null_mut())),
    uplink_handle (static_cast<handle_t> (null_mut())),
    rdata_notify_handle (static_cast<handle_t> (null_mut())),
    pending_notify_handle (static_cast<handle_t> (null_mut())),
    out_buffer (null_mut()),
    out_buffer_size (0),
    write_size (0)
{
    int rc = msg.init ();
    errno_assert (rc == 0);
}

int pgm_sender_t::init (udp_encapsulation_: bool, network_: &str)
{
    int rc = pgm_socket.init (udp_encapsulation_, network_);
    if (rc != 0)
        return rc;

    out_buffer_size = pgm_socket.get_max_tsdu_size ();
    out_buffer = (unsigned char *) malloc (out_buffer_size);
    alloc_assert (out_buffer);

    return rc;
}

void pgm_sender_t::plug (io_thread_t *io_thread_, session_base_t *session_)
{
    LIBZMQ_UNUSED (io_thread_);
    //  Allocate 2 fds for PGM socket.
    fd_t downlink_socket_fd = retired_fd;
    fd_t uplink_socket_fd = retired_fd;
    fd_t rdata_notify_fd = retired_fd;
    fd_t pending_notify_fd = retired_fd;

    session = session_;

    //  Fill fds from PGM transport and add them to the poller.
    pgm_socket.get_sender_fds (&downlink_socket_fd, &uplink_socket_fd,
                               &rdata_notify_fd, &pending_notify_fd);

    handle = add_fd (downlink_socket_fd);
    uplink_handle = add_fd (uplink_socket_fd);
    rdata_notify_handle = add_fd (rdata_notify_fd);
    pending_notify_handle = add_fd (pending_notify_fd);

    //  Set POLLIN. We will never want to stop polling for uplink = we never
    //  want to stop processing NAKs.
    set_pollin (uplink_handle);
    set_pollin (rdata_notify_handle);
    set_pollin (pending_notify_handle);

    //  Set POLLOUT for downlink_socket_handle.
    set_pollout (handle);
}

void pgm_sender_t::unplug ()
{
    if (has_rx_timer) {
        cancel_timer (rx_timer_id);
        has_rx_timer = false;
    }

    if (has_tx_timer) {
        cancel_timer (tx_timer_id);
        has_tx_timer = false;
    }

    rm_fd (handle);
    rm_fd (uplink_handle);
    rm_fd (rdata_notify_handle);
    rm_fd (pending_notify_handle);
    session = null_mut();
}

void pgm_sender_t::terminate ()
{
    unplug ();
    delete this;
}

void pgm_sender_t::restart_output ()
{
    set_pollout (handle);
    out_event ();
}

bool pgm_sender_t::restart_input ()
{
    zmq_assert (false);
    return true;
}

const EndpointUriPair &pgm_sender_t::get_endpoint () const
{
    return _empty_endpoint;
}

pgm_sender_t::~pgm_sender_t ()
{
    int rc = msg.close ();
    errno_assert (rc == 0);

    if (out_buffer) {
        free (out_buffer);
        out_buffer = null_mut();
    }
}

void pgm_sender_t::in_event ()
{
    if (has_rx_timer) {
        cancel_timer (rx_timer_id);
        has_rx_timer = false;
    }

    //  In-event on sender side means NAK or SPMR receiving from some peer.
    pgm_socket.process_upstream ();
    if (errno == ENOMEM || errno == EBUSY) {
        const long timeout = pgm_socket.get_rx_timeout ();
        add_timer (timeout, rx_timer_id);
        has_rx_timer = true;
    }
}

void pgm_sender_t::out_event ()
{
    //  POLLOUT event from send socket. If write buffer is empty,
    //  try to read new data from the encoder.
    if (write_size == 0) {
        //  First two bytes (sizeof uint16_t) are used to store message
        //  offset in following steps. Note that by passing our buffer to
        //  the get data function we prevent it from returning its own buffer.
        unsigned char *bf = out_buffer + mem::size_of::<uint16_t>();
        size_t bfsz = out_buffer_size - mem::size_of::<uint16_t>();
        uint16_t offset = 0xffff;

        size_t bytes = encoder.encode (&bf, bfsz);
        while (bytes < bfsz) {
            if (!more_flag && offset == 0xffff)
                offset = static_cast<uint16_t> (bytes);
            int rc = session->pull_msg (&msg);
            if (rc == -1)
                break;
            more_flag = msg.flags () & ZmqMessage::more;
            encoder.load_msg (&msg);
            bf = out_buffer + mem::size_of::<uint16_t>() + bytes;
            bytes += encoder.encode (&bf, bfsz - bytes);
        }

        //  If there are no data to write stop polling for output.
        if (bytes == 0) {
            reset_pollout (handle);
            return;
        }

        write_size = mem::size_of::<uint16_t>() + bytes;

        //  Put offset information in the buffer.
        put_uint16 (out_buffer, offset);
    }

    if (has_tx_timer) {
        cancel_timer (tx_timer_id);
        set_pollout (handle);
        has_tx_timer = false;
    }

    //  Send the data.
    size_t nbytes = pgm_socket.send (out_buffer, write_size);

    //  We can write either all data or 0 which means rate limit reached.
    if (nbytes == write_size)
        write_size = 0;
    else {
        zmq_assert (nbytes == 0);

        if (errno == ENOMEM) {
            // Stop polling handle and wait for tx timeout
            const long timeout = pgm_socket.get_tx_timeout ();
            add_timer (timeout, tx_timer_id);
            reset_pollout (handle);
            has_tx_timer = true;
        } else
            errno_assert (errno == EBUSY);
    }
}

void pgm_sender_t::timer_event (token: i32)
{
    //  Timer cancels on return by poller_base.
    if (token == rx_timer_id) {
        has_rx_timer = false;
        in_event ();
    } else if (token == tx_timer_id) {
        // Restart polling handle and retry sending
        has_tx_timer = false;
        set_pollout (handle);
        out_event ();
    } else
        zmq_assert (false);
}

// #endif
