use crate::address::ZmqAddress;
use crate::context::ZmqContext;
use crate::defines::ZmqHandle;
use crate::endpoint::EndpointUriPair;
use crate::fd::ZmqFileDesc;
use crate::io_object::ZmqIoObject;
use crate::session_base::ZmqSessionBase;
use crate::thread_context::ZmqThreadContext;

#[derive(Default,Debug,Clone)]
pub struct ZmqEngine<'a> {
    pub plugged: bool,
    pub session: ZmqSessionBase,
    pub io_object: ZmqIoObject,
    pub handle: ZmqHandle,
    pub fd: ZmqFileDesc,
    pub context: &'a mut ZmqContext<'a>,
    pub address: ZmqAddress,
    pub out_address: ZmqAddress,
    pub recv_enabled: bool,
    pub send_enabled: bool,
}

impl <'a> ZmqEngine <'a> {
    //  Indicate if the engine has an handshake stage.
    //  If engine has handshake stage, engine must call session.engine_ready when the handshake is complete.
    // virtual bool has_handshake_stage () = 0;
    pub fn has_handshake_state(&self) -> bool {
        todo!()
    }

    //  Plug the engine to the session.
    // virtual void Plug (ZmqIoThread *io_thread_, pub struct ZmqSessionBase *session_) = 0;
    pub fn plug(&mut self, io_thread: &mut ZmqThreadContext, session: &mut ZmqSessionBase) {
        todo!()
    }

    //  Terminate and deallocate the engine. Note that 'detached'
    //  events are not fired on termination.
    // virtual void terminate () = 0;
    pub fn terminate(&mut self) {
        todo!()
    }

    //  This method is called by the session to signalise that more
    //  messages can be written to the pipe.
    //  Returns false if the engine was deleted due to an error.
    //  TODO it is probably better to change the design such that the engine
    //  does not delete itself
    // virtual bool restart_input () = 0;
    pub fn restart_input(&mut self) -> bool {
        todo!()
    }

    //  This method is called by the session to signalise that there
    //  are messages to send available.
    // virtual void restart_output () = 0;
    pub fn restart_output(&mut self) {
        todo!()
    }

    // virtual void zap_msg_available () = 0;
    pub fn zap_msg_available(&mut self) {
        todo!()}

    // virtual const EndpointUriPair &get_endpoint () const = 0;
    pub fn get_endpoint(&mut self) -> &EndpointUriPair {
        todo!()
    }
}