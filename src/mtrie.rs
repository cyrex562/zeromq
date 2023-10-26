use crate::pipe::ZmqPipe;
use crate::generic_mtrie::GenericMtrie;

pub type ZmqMtrie = GenericMtrie<ZmqPipe>;
