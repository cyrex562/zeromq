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

//  Adapter for dbuffer, to plug it in instead of a queue for the sake
//  of implementing the conflate socket option, which, if set, makes
//  the receiving side to discard all incoming messages but the last one.
//
//  reader_awake flag is needed here to mimic ypipe delicate behaviour
//  around the reader being asleep (see 'c' pointer being NULL in ypipe.hpp)

use std::collections::VecDeque;

use crate::ypipe_base::YpipeBase;

// template <typename T> class YpipeConflate  : public YpipeBase<T>
#[derive(Default, Debug, Clone)]
pub struct YpipeConflate<T> {
    pub base: YpipeBase<T>,
    // protected:
    //     dbuffer_t<T> dbuffer;
    pub dbuffer: VecDeque<T>,
    pub reader_awake: bool,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (YpipeConflate)
}

impl YpipeConflate<T> {
    // public:
    //  Initialises the pipe.
    // YpipeConflate () : reader_awake (false) {}
    pub fn new<T>() -> Self {
        Self {
            dbuffer: VecDeque::new(),
            ..Default::default()
        }
    }

    //  Following function (write) deliberately copies uninitialised data
    //  when used with zmq_msg. Initialising the VSM body for
    //  non-VSM messages won't be good for performance.

    // #ifdef ZMQ_HAVE_OPENVMS
    // #pragma message save
    // #pragma message disable(UNINIT)
    // #endif
    pub fn write(&mut self, value_: &T, incomplete_: bool) {
        self.dbuffer.write(value_);
    }

    // #ifdef ZMQ_HAVE_OPENVMS
    // #pragma message restore
    // #endif

    // There are no incomplete items for conflate ypipe
    pub fn unwrite(&mut self) -> bool {
        return false;
    }

    //  Flush is no-op for conflate ypipe. Reader asleep behaviour
    //  is as of the usual ypipe.
    //  Returns false if the reader thread is sleeping. In that case,
    //  caller is obliged to wake the reader up before using the pipe again.
    pub fn flush(&mut self) -> bool {
        return self.reader_awake;
    }

    //  Check whether item is available for reading.
    pub fn check_read(&mut self) {
        let res = self.dbuffer.check_read();
        if (!res) {
            self.reader_awake = false;
        }

        return res;
    }

    //  Reads an item from the pipe. Returns false if there is no value.
    //  available.
    pub fn read(&mut self, value_: &T) -> bool {
        if (!check_read()) {
            return false;
        }

        return self.dbuffer.read(value_);
    }

    //  Applies the function fn to the first element in the pipe
    //  and returns the value returned by the fn.
    //  The pipe mustn't be empty or the function crashes.
    pub fn probe(&mut self, fn_: fn(a: &T) -> bool) {
        return self.dbuffer.probe(fn_);
    }
}

// #endif
