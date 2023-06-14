use std::ptr::null_mut;
use crate::v2_decoder::ZmqV2Decoder;

#[derive(Default, Debug, Clone)]
pub struct NormRxStreamState<'a> {
    pub norm_stream: NormObjectHandle,
    // i64 max_msg_size;
    pub max_msg_size: i64,
    pub zero_copy: bool,
    pub in_batch_size: i32,
    pub in_sync: bool,
    pub rx_ready: bool,
    pub zmq_decoder: Option<&'a mut ZmqV2Decoder>,
    pub skip_norm_sync: bool,
    pub buffer_ptr: Option<Vec<u8>>,
    pub buffer_size: usize,
    pub buffer_count: usize,
    // NormRxStreamState *prev;
    // NormRxStreamState *next;
    // NormRxStreamState::List *list;
}

impl NormRxStreamState {
    // List *AccessList () { return list; }

    // NormRxStreamState (NormObjectHandle normStream,
    // maxMsgSize: i64,
    // zeroCopy: bool,
    // inBatchSize: i32);
    // NormEngine::NormRxStreamState::NormRxStreamState
    pub fn new(
        normStream: NormObjectHandle,
        maxMsgSize: i64,
        zeroCopy: bool,
        inBatchSize: i32,
    ) -> Self {
        // norm_stream (normStream),
        //     max_msg_size (maxMsgSize),
        //     zero_copy (zeroCopy),
        //     in_batch_size (inBatchSize),
        //     in_sync (false),
        //     rx_ready (false),
        //     zmq_decoder (null_mut()),
        //     skip_norm_sync (false),
        //     buffer_ptr (null_mut()),
        //     buffer_size (0),
        //     buffer_count (0),
        //     prev (null_mut()),
        //     next (null_mut()),
        //     list (null_mut())
        Self {
            norm_stream: normStream,
            max_msg_size: maxMsgSize,
            zero_copy: zeroCopy,
            in_batch_size: inBatchSize,
            in_sync: false,
            rx_ready: false,
            zmq_decoder: None,
            skip_norm_sync: false,
            buffer_ptr: None,
            buffer_size: 0,
            buffer_count: 0,
            // prev: None,
            // next: None,
            // list: None,
        }
    }

    // ~NormRxStreamState ();

    // NormObjectHandle GetStreamHandle () const { return norm_stream; }

    // bool Init ();
    pub fn Init(&mut self) -> bool {
        in_sync = false;
        skip_norm_sync = false;
        if null_mut() != zmq_decoder {
            // delete
            // zmq_decoder;
        }
        zmq_decoder = ZmqV2Decoder(in_batch_size, max_msg_size, zero_copy);
        // alloc_assert (zmq_decoder);
        return if null_mut() != zmq_decoder {
            buffer_count = 0;
            buffer_size = 0;
            zmq_decoder.get_buffer(&buffer_ptr, &buffer_size);
            true
        } else {
            false
        };
    }

    //end NormEngine::NormRxStreamState::Init()

    // void SetRxReady (state: bool) { rx_ready = state; }

    // bool IsRxReady () const { return rx_ready; }

    // void SetSync (state: bool) { in_sync = state; }

    // bool InSync () const { return in_sync; }

    // These are used to feed data to decoder
    // and its underlying "msg" buffer
    // char *AccessBuffer () { return  (buffer_ptr + buffer_count); }

    // size_t GetBytesNeeded () const { return buffer_size - buffer_count; }

    // void IncrementBufferCount (count: usize) { buffer_count += count; }

    // ZmqMessage *AccessMsg () { return zmq_decoder.msg (); }

    // This invokes the decoder "decode" method
    // returning 0 if more data is needed,
    // 1 if the message is complete, If an error
    // occurs the 'sync' is dropped and the
    // decoder re-initialized
    // int Decode ();

    // This decodes any pending data sitting in our stream decoder buffer
    // It returns 1 upon message completion, -1 on error, 1 on msg completion
    pub fn Decode(&mut self) -> i32 {
        // If we have pending bytes to decode, process those first
        while buffer_count > 0 {
            // There's pending data for the decoder to decode
            let processed = 0;

            // This a bit of a kludgy approach used to weed
            // out the NORM ZMQ message transport "syncFlag" byte
            // from the ZMQ message stream being decoded (but it works!)
            if skip_norm_sync {
                buffer_ptr += 1;
                buffer_count -= 1;
                skip_norm_sync = false;
            }

            let rc = zmq_decoder.decode(buffer_ptr, buffer_count, processed);
            buffer_ptr += processed;
            buffer_count -= processed;
            match rc {
                1 => {
                    // msg completed
                    if 0 == buffer_count {
                        buffer_size = 0;
                        zmq_decoder.get_buffer(&buffer_ptr, &buffer_size);
                    }
                    skip_norm_sync = true;
                    return 1;
                }
                -1 => {
                    // decoder error (reset decoder and state variables)
                    in_sync = false;
                    skip_norm_sync = false; // will get consumed by norm sync check
                    Init();
                    // break;
                }
                0 => {} // need more data, keep decoding until buffer exhausted
                        // break;
            }
        }
        // Reset buffer pointer/count for next read
        buffer_count = 0;
        buffer_size = 0;
        zmq_decoder.get_buffer(&buffer_ptr, &buffer_size);
        return 0; //  need more data
    } // end NormEngine::NormRxStreamState::Decode()

}
