// #include "precompiled.hpp"

// #include "platform.hpp"

// #if defined ZMQ_HAVE_NORM

// #include "norm_engine.hpp"
// #ifdef ZMQ_USE_NORM_SOCKET_WRAPPER
// #include "ip.hpp"
// #endif

// #include "session_base.hpp"
// #include "v2_protocol.hpp"

use crate::defines::ZmqFileDesc;
use crate::defines::{NormDescriptor, NormInstanceHandle, ZmqHandle};
use crate::endpoint_uri::EndpointUriPair;
use crate::io_object::ZmqIoObject;
use crate::message::{ZmqMessage, ZMQ_MSG_MORE};

use crate::context::ZmqContext;
use crate::engine::ZmqEngine;
use crate::norm_stream_state::NormRxStreamState;
use crate::session_base::ZmqSessionBase;
use crate::thread_context::ZmqThreadContext;
use crate::v2_decoder::ZmqV2Decoder;
use crate::v2_encoder::ZmqV2Encoder;
use bincode::options;
use libc::{atoi, free, size_t, strchr, strncpy, strrchr, EAGAIN, EINVAL};
use std::mem;
use std::os::raw::c_void;
use std::path::Iter;
use std::ptr::null_mut;
use windows::Win32::Foundation::{HANDLE, LPARAM, WAIT_OBJECT_0, WPARAM};
use windows::Win32::Networking::WinSock::{closesocket, recv, send, shutdown, SOCKET};
use windows::Win32::System::Threading::{
    CreateThread, GetExitCodeThread, WaitForSingleObject, INFINITE,
};

// enum
// {
//     BUFFER_SIZE = 2048
// };
pub const BUFFER_SIZE: usize = 2048;

pub struct Iterator {
    // NormRxStreamState *next_item;
}

impl Iterator {
    // Iterator (const List &list);

    // NormRxStreamState *GetNextItem ();
}

pub struct List {
    // NormRxStreamState * head;

    // NormRxStreamState * tail;
} // end class NormEngine::NormRxStreamState::List

impl List {
    // List ();
    // ~List ();

    // void Append (NormRxStreamState &item);
    // void Remove (NormRxStreamState &item);

    // bool IsEmpty () const { return null_mut() == head; }

    // void Destroy ();
} // end class NormEngine::NormRxStreamState

#[derive(Default, Debug, Clone)]
pub struct NormWrapperThreadArgs {
    // NormDescriptor norm_descriptor;
    pub norm_descriptor: NormDescriptor,
    // SOCKET wrapper_write_fd;
    pub wrapper_write_fd: SOCKET,
    // NormInstanceHandle norm_instance_handle;
    pub norm_instance_handle: NormInstanceHandle,
}

// DWORD WINAPI normWrapperThread (LPVOID lpParam);
// #endif

// NormEngine::~NormEngine ()
// {
//     shutdown (); // in case it was not already called
// }

#[cfg(target_feature = "norm")]
pub fn norm_shutdown(engine: &mut ZmqEngine) {
    // TBD - implement a more graceful shutdown option
    if (is_receiver) {
        NormStopReceiver(norm_session);

        // delete any active NormRxStreamState
        rx_pending_list.Destroy();
        rx_ready_list.Destroy();
        msg_ready_list.Destroy();

        is_receiver = false;
    }
    if (is_sender) {
        NormStopSender(norm_session);
        is_sender = false;
    }
    if (NORM_SESSION_INVALID != norm_session) {
        NormDestroySession(norm_session);
        norm_session = NORM_SESSION_INVALID;
    }
    if (NORM_INSTANCE_INVALID != norm_instance) {
        NormStopInstance(norm_instance);
        NormDestroyInstance(norm_instance);
        norm_instance = NORM_INSTANCE_INVALID;
    }
} // end NormEngine::shutdown()

#[cfg(target_feature = "norm")]
pub fn norm_init(engine: &mut ZmqEngine, network_: &str, send: bool, recv: bool) -> i32 {
    // Parse the "network_" address int "iface", "addr", and "port"
    // norm endpoint format: [id,][<iface>;]<addr>:<port>
    // First, look for optional local NormNodeId
    // (default NORM_NODE_ANY causes NORM to use host IP addr for NormNodeId)
    let localId = NORM_NODE_ANY;
    // const char *ifacePtr = strchr (network_, ',');
    let mut ifacePtr = network_.find(',');
    if ifacePtr.is_some() {
        let idLen = ifacePtr - network_;
        if (idLen > 31) {
            idLen = 31;
        }
        let mut idText: [u8; 32];
        // strncpy (idText, network_, idLen);
        idText[..idLen].copy_from_slice(&network_[..idLen]);
        // idText[idLen] = 0;
        let localId = i32::from_str_radix(String::from_utf8_lossy(&idText).as_ref(), 10).unwrap();
        ifacePtr += 1;
    } else {
        // ifacePtr = network_;
    }

    // Second, look for optional multicast ifaceName
    let mut ifaceName: [u8; 256] = [0; 256];
    let addrPtr = ifacePtr.find(';'); //strchr (ifacePtr, ';');
    if addrPtr.is_some() {
        let ifaceLen = addrPtr - ifacePtr;
        if (ifaceLen > 255) {
            ifaceLen = 255;
        } // return error instead?
          // strncpy (ifaceName, ifacePtr, ifaceLen);
        ifaceName[..ifaceLen].copy_from_slice(&ifacePtr[..ifaceLen]);
        ifaceName[ifaceLen] = 0;
        // ifacePtr = ifaceName;
        addrPtr += 1;
    } else {
        addrPtr = ifacePtr;
        // ifacePtr = null_mut();
    }

    // Finally, parse IP address and port number
    let portPtr = addrPtr.find(':'); // strrchr (addrPtr, ':');
    if (portPtr.is_some()) {
      // errno = EINVAL;
        return -1;
    }

    // char addr[256];
    let mut addr = [0u8; 256];
    let mut addrLen = portPtr - addrPtr;
    if (addrLen > 255) {
        addrLen = 255;
    }
    // strncpy (addr, addrPtr, addrLen);
    addr = addrPtr[..addrLen].try_into().unwrap();
    addr[addrLen] = 0;
    portPtr += 1;
    let portNumber = i32::from_str_radix(String::from_utf8_lossy(portPtr).as_ref(), 10).unwrap();

    if (NORM_INSTANCE_INVALID == norm_instance) {
        if (NORM_INSTANCE_INVALID == (norm_instance = NormCreateInstance())) {
            // errno set by whatever caused NormCreateInstance() to fail
            return -1;
        }
    }

    // TBD - What do we use for our local NormNodeId?
    //       (for now we use automatic, IP addr based assignment or passed in 'id')
    //       a) Use ZMQ Identity somehow?
    //       b) Add function to use iface addr
    //       c) Randomize and implement a NORM session layer
    //          conflict detection/resolution protocol

    norm_session = NormCreateSession(norm_instance, addr, portNumber, localId);
    if (NORM_SESSION_INVALID == norm_session) {
        let savedErrno = errno;
        NormDestroyInstance(norm_instance);
        norm_instance = NORM_INSTANCE_INVALID;
      // errno = savedErrno;
        return -1;
    }
    // There's many other useful NORM options that could be applied here
    if (NormIsUnicastAddress(addr)) {
        NormSetDefaultUnicastNack(norm_session, true);
    } else {
        // These only apply for multicast sessions
        //NormSetTTL(norm_session, options.multicast_hops);  // ZMQ default is 1
        NormSetTTL(norm_session, 255); // since the ZMQ_MULTICAST_HOPS socket option isn't well-supported
        NormSetRxPortReuse(norm_session, true); // port reuse doesn't work for non-Connected unicast
        NormSetLoopback(norm_session, true); // needed when multicast users on same machine
        if (ifacePtr.is_some()) {
            // Note a bad interface may not be caught until sender or receiver start
            // (Since sender/receiver is not yet started, this always succeeds here)
            NormSetMulticastInterface(norm_session, ifacePtr);
        }
    }

    if (recv) {
        // The alternative NORM_SYNC_CURRENT here would provide "instant"
        // receiver sync to the sender's _current_ message transmission.
        // NORM_SYNC_STREAM tries to get everything the sender has cached/buffered
        NormSetDefaultSyncPolicy(norm_session, NORM_SYNC_STREAM);
        if (!NormStartReceiver(norm_session, 2 * 1024 * 1024)) {
            // errno set by whatever failed
            let savedErrno = errno;
            NormDestroyInstance(norm_instance); // session gets closed, too
            norm_session = NORM_SESSION_INVALID;
            norm_instance = NORM_INSTANCE_INVALID;
          // errno = savedErrno;
            return -1;
        }
        is_receiver = true;
    }

    if (send) {
        // Pick a random sender instance id (aka norm sender session id)
        let instanceId = NormGetRandomSessionId();
        // TBD - provide "options" for some NORM sender parameters
        if (!NormStartSender(norm_session, instanceId, 2 * 1024 * 1024, 1400, 16, 4)) {
            // errno set by whatever failed
            let savedErrno = errno;
            NormDestroyInstance(norm_instance); // session gets closed, too
            norm_session = NORM_SESSION_INVALID;
            norm_instance = NORM_INSTANCE_INVALID;
          // errno = savedErrno;
            return -1;
        }
        NormSetCongestionControl(norm_session, true);
        norm_tx_ready = true;
        is_sender = true;
        if (NORM_OBJECT_INVALID == (norm_tx_stream = NormStreamOpen(norm_session, 2 * 1024 * 1024)))
        {
            // errno set by whatever failed
            let savedErrno = errno;
            NormDestroyInstance(norm_instance); // session gets closed, too
            norm_session = NORM_SESSION_INVALID;
            norm_instance = NORM_INSTANCE_INVALID;
          // errno = savedErrno;
            return -1;
        }
    }

    //NormSetMessageTrace(norm_session, true);
    //NormSetDebugLevel(3);
    //NormOpenDebugLog(norm_instance, "normLog.txt");

    return 0; // no error
} // end NormEngine::init()

//  ZmqIEngine interface implementation.
//  Plug the engine to the session.
// void Plug (ZmqIoThread *io_thread_, ZmqSessionBase *session_) ;
#[cfg(target_feature = "norm")]
pub fn norm_plug(
    engine: &mut ZmqEngine,
    io_thread: &mut ZmqThreadContext,
    session: &mut ZmqSessionBase,
) {
    // #ifdef ZMQ_USE_NORM_SOCKET_WRAPPER
    let mut threadArgs = NormWrapperThreadArgs::default();
    let rc = make_fdpair(
        &mut wrapper_read_fd,
        &mut threadArgs.wrapper_write_fd as &mut ZmqFileDesc,
    );
    // errno_assert (rc != -1);

    threadArgs.norm_descriptor = NormGetDescriptor(norm_instance);
    threadArgs.norm_instance_handle = norm_instance;
    norm_descriptor_handle = add_fd(wrapper_read_fd);
    // #else
    let normDescriptor = NormGetDescriptor(norm_instance);
    norm_descriptor_handle = add_fd(normDescriptor);
    // #endif
    // Set POLLIN for notification of pending NormEvents
    set_pollin(norm_descriptor_handle);
    // TBD - we may assign the NORM engine to an io_thread in the future???
    zmq_session = session_;
    if (is_sender) {
        zmq_output_ready = true;
    }
    if (is_receiver) {
        zmq_input_ready = true;
    }

    if (is_sender) {
        send_data();
    }

    // #ifdef ZMQ_USE_NORM_SOCKET_WRAPPER
    // TODO
    //         unsafe {
    //             wrapper_thread_handle = CreateThread(null_mut(), 0, normWrapperThread,
    //                                                  threadArgs, 0, &wrapper_thread_id);
    //         }
    // #endif
} // end NormEngine::init()

//  Terminate and deallocate the engine. Note that 'detached'
//  events are not fired on termination.
// void terminate () ;
#[cfg(target_feature = "norm")]
pub fn norm_terminate(engine: &mut ZmqEngine) {
    self.unplug();
    self.shutdown();
    // delete this;
}

//  This method is called by the session to signalise that more
//  messages can be written to the pipe.
// bool restart_input () ;
#[cfg(target_feature = "norm")]
pub fn norm_restart_input(engine: &mut ZmqEngine) -> bool {
    // TBD - should we check/assert that zmq_input_ready was false???
    zmq_input_ready = true;
    // Process any pending received messages
    if !msg_ready_list.IsEmpty() {
        self.recv_data(NORM_OBJECT_INVALID);
    }

    return true;
}

//  This method is called by the session to signalise that there
//  are messages to send available.
// void restart_output () ;
#[cfg(target_feature = "norm")]
pub fn norm_restart_output(engine: &mut ZmqEngine) {
    // There's new message data available from the session
    zmq_output_ready = true;
    if (norm_tx_ready) {
        send_data();
    }
}

// void zap_msg_available ()  {}

// const EndpointUriPair &get_endpoint () const ;

// i_poll_events interface implementation.
// (we only need in_event() for NormEvent notification)
// (i.e., don't have any output events or timers (yet))
// void in_event ();
#[cfg(target_feature = "norm")]
pub fn norm_in_event(engine: &mut ZmqEngine) {
    // This means a NormEvent is pending, so call NormGetNextEvent() and handle
    let event = NormEvent::default();
    // #ifdef ZMQ_USE_NORM_SOCKET_WRAPPER
    let rc = self.recv(wrapper_read_fd, (&event), mem::size_of::<event>(), 0);
    // errno_assert (rc == mem::size_of::<event>());
    // #else
    if !NormGetNextEvent(norm_instance, &event) {
        // NORM has died before we unplugged?!
        // zmq_assert (false);
        return;
    }
    // #endif

    match event.type_ {
        NORM_TX_QUEUE_VACANCY | NORM_TX_QUEUE_EMPTY => {
            if (!norm_tx_ready) {
                norm_tx_ready = true;
                self.send_data();
            }
        }
        NORM_RX_OBJECT_NEW => {}
        NORM_RX_OBJECT_UPDATED => {
            self.recv_data(event.object);
        }
        NORM_RX_OBJECT_ABORTED => {
            NormRxStreamState * rxState = NormObjectGetUserData(event.object);
            if (null_mut() != rxState) {
                // Remove the state from the list it's in
                // This is now unnecessary since deletion takes care of list removal
                // but in the interest of being clear ...
                NormRxStreamState::List * list = rxState.AccessList();
                if (null_mut() != list) {
                    list.Remove(*rxState);
                }
            }
            // delete rxState;
        }
        // break;
        NORM_REMOTE_SENDER_INACTIVE => {
            // Here we free resources used for this formerly active sender.
            // Note w/ NORM_SYNC_STREAM, if sender reactivates, we may
            //  get some messages delivered twice.  NORM_SYNC_CURRENT would
            // mitigate that but might miss data at startup. Always tradeoffs.
            // Instead of immediately deleting, we could instead initiate a
            // user configurable timeout here to wait some amount of time
            // after this event to declare the remote sender truly dead
            // and delete its state???
            NormNodeDelete(event.sender);
        }
        _ => {} // We ignore some NORM events
                // break;
    }
} // NormEngine::in_event()

//
// void unplug ();
#[cfg(target_feature = "norm")]
pub fn norm_unplug(engine: &mut ZmqEngine) {
    rm_fd(norm_descriptor_handle);
    // #ifdef ZMQ_USE_NORM_SOCKET_WRAPPER
    PostThreadMessage(wrapper_thread_id, WM_QUIT, null_mut(), null_mut());
    unsafe {
        WaitForSingleObject(wrapper_thread_handle, INFINITE);
    }
    let mut exitCode = 0u32;
    unsafe {
        GetExitCodeThread(wrapper_thread_handle, &mut exitCode);
    }
    // zmq_assert (exitCode != -1);
    let rc = unsafe { closesocket(wrapper_read_fd) };
    // errno_assert (rc != -1);
    // #endif
    zmq_session = null_mut();
}

// void send_data ();
#[cfg(target_feature = "norm")]
pub fn norm_send_data(engine: &mut ZmqEngine) {
    // Here we write as much as is available or we can
    while (zmq_output_ready && norm_tx_ready) {
        if (0 == tx_len) {
            // Our tx_buffer needs data to send
            // Get more data from encoder
            let mut space = BUFFER_SIZE;
            let bufPtr = tx_buffer;
            tx_len = zmq_encoder.encode(&bufPtr, space);
            if (0 == tx_len) {
                if (tx_first_msg) {
                    // We don't need to mark eom/flush until a message is sent
                    tx_first_msg = false;
                } else {
                    // A prior message was completely written to stream, so
                    // mark end-of-message and possibly flush (to force packet transmission,
                    // even if it's not a full segment so message gets delivered quickly)
                    // NormStreamMarkEom(norm_tx_stream);  // the flush below marks eom
                    // Note NORM_FLUSH_ACTIVE makes NORM fairly chatty for low duty cycle messaging
                    // but makes sure content is delivered quickly.  Positive acknowledgements
                    // with flush override would make NORM more succinct here
                    NormStreamFlush(norm_tx_stream, true, NORM_FLUSH_ACTIVE);
                }
                // Need to pull and load a new message to send
                if -1 == zmq_session.pull_msg(&tx_msg) {
                    // We need to wait for "restart_output()" to be called by ZMQ
                    zmq_output_ready = false;
                    break;
                }
                zmq_encoder.load_msg(&tx_msg);
                // Should we write message size header for NORM to use? Or expect NORM
                // receiver to decode ZMQ message framing format(s)?
                // OK - we need to use a byte to denote when the ZMQ frame is the _first_
                //      frame of a message so it can be decoded properly when a receiver
                //      'syncs' mid-stream.  We key off the the state of the 'more_flag'
                //      I.e.,If  more_flag _was_ false previously, this is the first
                //      frame of a ZMQ message.
                if (tx_more_bit) {
                    tx_buffer[0] = 0xff;
                }
                // this is not first frame of message
                else {
                    tx_buffer[0] = 0x00;
                } // this is first frame of message
                tx_more_bit = (0 != (tx_msg.flags() & ZMQ_MSG_MORE));
                // Go ahead an get a first chunk of the message
                bufPtr += 1;
                space -= 1;
                tx_len = 1 + zmq_encoder.encode(&bufPtr, space);
                tx_index = 0;
            }
        }
        // Do we have data in our tx_buffer pending
        if (tx_index < tx_len) {
            // We have data in our tx_buffer to send, so write it to the stream
            tx_index += NormStreamWrite(norm_tx_stream, tx_buffer + tx_index, tx_len - tx_index);
            if (tx_index < tx_len) {
                // NORM stream buffer full, wait for NORM_TX_QUEUE_VACANCY
                norm_tx_ready = false;
                break;
            }
            tx_len = 0; // all buffered data was written
        }
    } // end while (zmq_output_ready && norm_tx_ready)
}

// void recv_data (NormObjectHandle stream);
#[cfg(target_feature = "norm")]
pub fn norm_recv_data(engine: &mut ZmqEngine, object: NormObjectHandle) {
    if NORM_OBJECT_INVALID != object {
        // Call result of NORM_RX_OBJECT_UPDATED notification
        // This is a rx_ready indication for a new or existing rx stream
        // First, determine if this is a stream we already know
        // zmq_assert (NORM_OBJECT_STREAM == NormObjectGetType (object));
        // Since there can be multiple senders (publishers), we keep
        // state for each separate rx stream.
        let rxState = NormObjectGetUserData(object);
        if null_mut() == rxState {
            // This is a new stream, so create rxState with zmq decoder, etc
            rxState = NormRxStreamState(
                object,
                options.maxmsgsize,
                options.zero_copy,
                options.in_batch_size,
            );
            // errno_assert (rxState);

            if !rxState.Init() {
                // errno_assert (false);
                // delete rxState;
                return;
            }
            NormObjectSetUserData(object, rxState);
        } else if !rxState.IsRxReady() {
            // Existing non-ready stream, so remove from pending
            // list to be promoted to rx_ready_list ...
            rx_pending_list.Remove(*rxState);
        }
        if !rxState.IsRxReady() {
            // TBD - prepend up front for immediate service?
            rxState.SetRxReady(true);
            rx_ready_list.Append(*rxState);
        }
    }
    // This loop repeats until we've read all data available from "rx ready" inbound streams
    // and pushed any accumulated messages we can up to the zmq session.
    while !rx_ready_list.IsEmpty() || (zmq_input_ready && !msg_ready_list.IsEmpty()) {
        // Iterate through our rx_ready streams, reading data into the decoder
        // (This services incoming "rx ready" streams in a round-robin fashion)
        // NormRxStreamState::List::Iterator iterator (rx_ready_list);
        NormRxStreamState * rxState;
        rxState = iterator.GetNextItem();
        while (null_mut() != rxState) {
            match rxState.Decode() {
                1 => {
                    // msg completed
                    // Complete message decoded, move this stream to msg_ready_list
                    // to push the message up to the session below.  Note the stream
                    // will be returned to the "rx_ready_list" after that's Done
                    rx_ready_list.Remove(*rxState);

                    msg_ready_list.Append(*rxState);
                    continue;
                }
                -1 => {
                    // decoding error (shouldn't happen w/ NORM, but ...)
                    // We need to re-sync this stream (decoder buffer was reset)
                    rxState.SetSync(false);
                }

                _ => {} // 0 - need more data
            }
            // Get more data from this stream
            let stream = rxState.GetStreamHandle();
            // First, make sure we're in sync ...
            while (!rxState.InSync()) {
                // seek NORM message start
                if (!NormStreamSeekMsgStart(stream)) {
                    // Need to wait for more data
                    break;
                }
                // read message 'flag' byte to see if this it's a 'final' frame
                let mut syncFlag = 0u8;
                let mut numBytes = 1;
                if (!NormStreamRead(stream, &syncFlag, &numBytes)) {
                    // broken stream (shouldn't happen after seek msg start?)
                    // zmq_assert (false);
                    continue;
                }
                if (0 == numBytes) {
                    // This probably shouldn't happen either since we found msg start
                    // Need to wait for more data
                    break;
                }
                if (0 == syncFlag) {
                    rxState.SetSync(true);
                }
                // else keep seeking ...
            } // end while(!rxState->InSync())
            if (!rxState.InSync()) {
                // Need more data for this stream, so remove from "rx ready"
                // list and iterate to next "rx ready" stream
                rxState.SetRxReady(false);
                // Move from rx_ready_list to rx_pending_list
                rx_ready_list.Remove(*rxState);
                rx_pending_list.Append(*rxState);
                continue;
            }
            // Now we're actually ready to read data from the NORM stream to the zmq_decoder
            // the underlying zmq_decoder->get_buffer() call sets how much is needed.
            let numBytes = rxState.GetBytesNeeded();
            if !NormStreamRead(stream, rxState.AccessBuffer(), &numBytes) {
                // broken NORM stream, so re-sync
                rxState.Init(); // TBD - check result
                                // This will retry syncing, and getting data from this stream
                                // since we don't increment the "it" iterator
                continue;
            }
            rxState.IncrementBufferCount(numBytes);
            if (0 == numBytes) {
                // All the data available has been read
                // Need to wait for NORM_RX_OBJECT_UPDATED for this stream
                rxState.SetRxReady(false);
                // Move from rx_ready_list to rx_pending_list
                rx_ready_list.Remove(*rxState);
                rx_pending_list.Append(*rxState);
            }
            rxState = iterator.GetNextItem();
        } // end while(NULL != (rxState = iterator.GetNextItem()))

        if zmq_input_ready {
            // At this point, we've made a pass through the "rx_ready" stream list
            // Now make a pass through the "msg_pending" list (if the zmq session
            // ready for more input).  This may possibly return streams back to
            // the "rx ready" stream list after their pending message is handled
            // NormRxStreamState::List::Iterator iterator (msg_ready_list);
            NormRxStreamState * rxState;
            rxState = iterator.GetNextItem();
            while (null_mut() != rxState) {
                ZmqMessage * msg = rxState.AccessMsg();
                let rc = zmq_session.push_msg(msg);
                if (-1 == rc) {
                    if (EAGAIN == errno) {
                        // need to wait until session calls "restart_input()"
                        zmq_input_ready = false;
                        break;
                    } else {
                        // session rejected message?
                        // TBD - handle this better
                        // zmq_assert (false);
                    }
                }
                // else message was accepted.
                msg_ready_list.Remove(*rxState);
                if rxState.IsRxReady() {
                    // Move back to "rx_ready" list to read more data
                    rx_ready_list.Append(*rxState);
                } else {
                    // Move back to "rx_pending" list until NORM_RX_OBJECT_UPDATED
                    msg_ready_list.Append(*rxState);
                }
                rxState = iterator.GetNextItem()
            } // end while(NULL != (rxState = iterator.GetNextItem()))
        } // end if (zmq_input_ready)
    } // end while ((!rx_ready_list.empty() || (zmq_input_ready && !msg_ready_list.empty()))

    // Alert zmq of the messages we have pushed up
    zmq_session.flush();
} // end NormEngine::recv_data()
