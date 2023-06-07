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

// #ifndef __ZMQ_CURVE_CLIENT_TOOLS_HPP_INCLUDED__
// #define __ZMQ_CURVE_CLIENT_TOOLS_HPP_INCLUDED__

// #ifdef ZMQ_HAVE_CURVE

// #if defined(ZMQ_USE_TWEETNACL)
// #include "tweetnacl.h"
// #elif defined(ZMQ_USE_LIBSODIUM)
// #include "sodium.h"
// #endif

// #if CRYPTO_BOX_NONCEBYTES != 24 || CRYPTO_BOX_PUBLICKEYBYTES != 32             \
//   || CRYPTO_BOX_SECRETKEYBYTES != 32 || CRYPTO_BOX_ZEROBYTES != 32             \
//   || CRYPTO_BOX_BOXZEROBYTES != 16
// #error "CURVE library not built properly"
// #endif

// #include "wire.hpp"
// #include "err.hpp"
// #include "secure_allocator.hpp"

// #include <vector>

use crate::config::{CRYPTO_BOX_BOXZEROBYTES, CRYPTO_BOX_NONCEBYTES, CRYPTO_BOX_ZEROBYTES};
use crate::utils::copy_bytes;
use anyhow::anyhow;

pub fn process_welcome(
    msg_data_: &[u8],
    msg_size_: usize,
    server_key_: &[u8],
    cn_secret_: &[u8],
    cn_server_: &mut [u8],
    cn_cookie_: &mut [u8],
    cn_precom_: &mut [u8],
) -> anyhow::Result<()> {
    if (msg_size_ != 168) {
        // errno = EPROTO;
        // return -1;
        return Err(anyhow!("EPROTO"));
    }

    let mut welcome_nonce: [u8; CRYPTO_BOX_NONCEBYTES as usize] = [0; CRYPTO_BOX_NONCEBYTES];
    // std::vector<uint8_t, secure_allocator_t<uint8_t> > welcome_plaintext (
    //   CRYPTO_BOX_ZEROBYTES + 128);
    let mut welcome_plaintext: Vec<u8> = Vec::with_capacity(CRYPTO_BOX_ZEROBYTES as usize);
    welcome_plaintext.fill(0);
    let mut welcome_box: [u8; (CRYPTO_BOX_BOXZEROBYTES + 144) as usize] =
        [0; CRYPTO_BOX_BOXZEROBYTES + 144];

    //  Open Box [S' + cookie](C'->S)
    // memset (welcome_box, 0, CRYPTO_BOX_BOXZEROBYTES);
    // memcpy (welcome_box + CRYPTO_BOX_BOXZEROBYTES, msg_data_ + 24, 144);
    copy_bytes(
        &mut welcome_box,
        CRYPTO_BOX_BOXZEROBYTES as usize,
        msg_data_,
        24,
        144,
    );

    // memcpy (welcome_nonce, "WELCOME-", 8);
    copy_bytes(&mut welcome_nonce, 0, b"WELCOME-", 0, 8);
    // memcpy (welcome_nonce + 8, msg_data_ + 8, 16);
    copy_bytes(&mut welcome_nonce, 8, msg_data_, 8, 16);

    crypto_box_open(
        &welcome_plaintext[0],
        welcome_box,
        welcome_box.len(),
        welcome_nonce,
        server_key_,
        cn_secret_,
    )?;
    // if (rc != 0) {
    //     errno = EPROTO;
    //     return -1;
    // }

    // memcpy (cn_server_, &welcome_plaintext[CRYPTO_BOX_ZEROBYTES], 32);
    copy_bytes(
        cn_server_,
        0,
        &welcome_plaintext,
        CRYPTO_BOX_ZEROBYTES as usize,
        32,
    );

    // memcpy (cn_cookie_, &welcome_plaintext[CRYPTO_BOX_ZEROBYTES + 32],
    //         16 + 80);
    copy_bytes(
        cn_cookie_,
        0,
        &welcome_plaintext,
        (CRYPTO_BOX_ZEROBYTES + 32) as usize,
        16 + 80,
    );

    //  Message independent precomputation
    crypto_box_beforenm(cn_precom_, cn_server_, cn_secret_)?;
    // zmq_assert (rc == 0);

    // return 0;
    Ok(())
}

pub fn produce_initiate(
    data: &mut [u8],
    size: usize,
    cn_nonce_: u64,
    server_key_: &[u8],
    public_key_: &[u8],
    secret_key_: &[u8],
    cn_public_: &[u8],
    cn_secret_: &[u8],
    cn_server_: &[u8],
    cn_cookie_: &[u8],
    metadata_plaintext_: &[u8],
    metadata_length_: usize,
) -> anyhow::Result<()> {
    let mut vouch_nonce: [u8; CRYPTO_BOX_NONCEBYTES as usize] = [0; CRYPTO_BOX_NONCEBYTES];
    // std::vector<uint8_t, secure_allocator_t<uint8_t> > vouch_plaintext (
    //   CRYPTO_BOX_ZEROBYTES + 64);
    let mut vouch_plaintext: Vec<u8> = Vec::with_capacity((CRYPTO_BOX_ZEROBYTES + 64) as usize);
    let mut vouch_box: [u8; (CRYPTO_BOX_BOXZEROBYTES + 80) as usize] =
        [0; CRYPTO_BOX_BOXZEROBYTES + 80];

    //  Create vouch = Box [C',S](C->S')
    // std::fill (vouch_plaintext.begin (),
    //            vouch_plaintext.begin () + CRYPTO_BOX_ZEROBYTES, 0);
    vouch_plaintext.fill(0);

    // memcpy (&vouch_plaintext[CRYPTO_BOX_ZEROBYTES], cn_public_, 32);
    copy_bytes(
        &mut vouch_plaintext,
        CRYPTO_BOX_ZEROBYTES as usize,
        cn_public_,
        0,
        32,
    );

    // memcpy (&vouch_plaintext[CRYPTO_BOX_ZEROBYTES + 32], server_key_, 32);
    copy_bytes(
        &mut vouch_plaintext,
        (CRYPTO_BOX_ZEROBYTES + 32) as usize,
        server_key_,
        0,
        32,
    );

    // memset (vouch_nonce, 0, CRYPTO_BOX_NONCEBYTES);

    // memcpy (vouch_nonce, "VOUCH---", 8);
    copy_bytes(&mut vouch_nonce, 0, b"VOUCH---", 0, 8);
    randombytes(vouch_nonce + 8, 16);

    crypto_box(
        vouch_box,
        &vouch_plaintext[0],
        vouch_plaintext.size(),
        vouch_nonce,
        cn_server_,
        secret_key_,
    )?;
    // if (rc == -1)
    //     return -1;

    let mut initiate_nonce: [u8; CRYPTO_BOX_NONCEBYTES as usize] = [0; CRYPTO_BOX_NONCEBYTES];

    // std::vector<uint8_t> initiate_box (CRYPTO_BOX_BOXZEROBYTES + 144
    //                                    + metadata_length_);
    let mut initiate_box: Vec<u8> =
        Vec::with_capacity((CRYPTO_BOX_BOXZEROBYTES + 144 + metadata_length_) as usize);
    initiate_box.fill(0);

    // std::vector<uint8_t, secure_allocator_t<uint8_t> > initiate_plaintext (
    //   CRYPTO_BOX_ZEROBYTES + 128 + metadata_length_);
    let mut initiate_plaintext: Vec<u8> =
        Vec::with_capacity((CRYPTO_BOX_ZEROBYTES + 128 + metadata_length_) as usize);
    initiate_plaintext.fill(0);

    //  Create Box [C + vouch + metadata](C'->S')
    // std::fill (initiate_plaintext.begin (),
    //            initiate_plaintext.begin () + CRYPTO_BOX_ZEROBYTES, 0);

    //  False positives due to https://gcc.gnu.org/bugzilla/show_bug.cgi?id=99578
    // #if __GNUC__ >= 11
    // #pragma GCC diagnostic ignored "-Warray-bounds"
    // #pragma GCC diagnostic ignored "-Wstringop-overflow="
    // #endif
    //     memcpy (&initiate_plaintext[CRYPTO_BOX_ZEROBYTES], public_key_, 32);
    copy_bytes(
        &mut initiate_plaintext,
        CRYPTO_BOX_ZEROBYTES as usize,
        public_key_,
        0,
        32,
    );

    // memcpy (&initiate_plaintext[CRYPTO_BOX_ZEROBYTES + 32], vouch_nonce + 8,
    //         16);
    copy_bytes(
        &mut initiate_plaintext,
        (CRYPTO_BOX_ZEROBYTES + 32) as usize,
        &vouch_nonce,
        8,
        16,
    );

    // memcpy (&initiate_plaintext[CRYPTO_BOX_ZEROBYTES + 48],
    //         vouch_box + CRYPTO_BOX_BOXZEROBYTES, 80);
    copy_bytes(
        &mut initiate_plaintext,
        (CRYPTO_BOX_ZEROBYTES + 48) as usize,
        &vouch_box,
        CRYPTO_BOX_ZEROBYTES as usize,
        80,
    );

    if (metadata_length_) {
        // memcpy (&initiate_plaintext[CRYPTO_BOX_ZEROBYTES + 48 + 80],
        //         metadata_plaintext_, metadata_length_);
        copy_bytes(
            &mut initiate_plaintext,
            (CRYPTO_BOX_ZEROBYTES + 48 + 80) as usize,
            metadata_plaintext_,
            0,
            metadata_length_,
        );
    }
    // #if __GNUC__ >= 11
    // #pragma GCC diagnostic pop
    // #pragma GCC diagnostic pop
    // #endif

    // memcpy (initiate_nonce, "CurveZMQINITIATE", 16);
    copy_bytes(&mut initiate_nonce, 0, b"CurveZMQINITIATE", 0, 16);
    put_uint64(initiate_nonce + 16, cn_nonce_);

    crypto_box(
        &initiate_box[0],
        &initiate_plaintext[0],
        CRYPTO_BOX_ZEROBYTES + 128 + metadata_length_,
        initiate_nonce,
        cn_server_,
        cn_secret_,
    )?;

    // if (rc == -1)
    //     return -1;

    // uint8_t *initiate = static_cast<uint8_t *> (data);
    let initiate: &mut [u8] = data;

    // zmq_assert (size
    //             == 113 + 128 + CRYPTO_BOX_BOXZEROBYTES + metadata_length_);

    // memcpy (initiate, "\x08INITIATE", 9);
    copy_bytes(initiate, 0, b"\x08INITIATE", 0, 9);
    //  Cookie provided by the server in the WELCOME command
    // memcpy (initiate + 9, cn_cookie_, 96);
    copy_bytes(initiate, 9, cn_cookie_, 0, 96);
    //  Short nonce, prefixed by "CurveZMQINITIATE"
    // memcpy (initiate + 105, initiate_nonce + 16, 8);
    copy_bytes(initiate, 105, &initiate_nonce, 16, 8);
    //  Box [C + vouch + metadata](C'->S')
    // memcpy (initiate + 113, &initiate_box[CRYPTO_BOX_BOXZEROBYTES],
    //         128 + metadata_length_ + CRYPTO_BOX_BOXZEROBYTES);
    copy_bytes(
        initiate,
        113,
        &initiate_box,
        CRYPTO_BOX_BOXZEROBYTES as usize,
        128 + metadata_length_ + CRYPTO_BOX_BOXZEROBYTES,
    );

    // return 0;
    Ok(())
}

pub fn is_handshake_command_welcome(msg_data_: &[u8], msg_size_: usize) -> bool {
    return is_handshake_command(msg_data_, msg_size_, b"\x07WELCOME");
}

pub fn is_handshake_command_ready(msg_data_: &[u8], msg_size_: usize) -> bool {
    return is_handshake_command(msg_data_, msg_size_, b"\x05READY");
}

pub fn is_handshake_command_error(msg_data_: &[u8], msg_size_: usize) -> bool {
    return is_handshake_command(msg_data_, msg_size_, b"\x05ERROR");
}

//  non-static functions
// curve_client_tools_t (
//   const uint8_t (&curve_public_key_)[CRYPTO_BOX_PUBLICKEYBYTES],
//   const uint8_t (&curve_secret_key_)[CRYPTO_BOX_SECRETKEYBYTES],
//   const uint8_t (&curve_server_key_)[CRYPTO_BOX_PUBLICKEYBYTES])
// {
//     rc: i32;
//     memcpy (public_key, curve_public_key_, CRYPTO_BOX_PUBLICKEYBYTES);
//     memcpy (secret_key, curve_secret_key_, CRYPTO_BOX_SECRETKEYBYTES);
//     memcpy (server_key, curve_server_key_, CRYPTO_BOX_PUBLICKEYBYTES);
//
//     //  Generate short-Term key pair
//     memset (cn_secret, 0, CRYPTO_BOX_SECRETKEYBYTES);
//     memset (cn_public, 0, CRYPTO_BOX_PUBLICKEYBYTES);
//     rc = crypto_box_keypair (cn_public, cn_secret);
//     zmq_assert (rc == 0);
// }

// int produce_hello (data: *mut c_void, const u64 cn_nonce_) const
// {
//     return produce_hello (data, server_key, cn_nonce_, cn_public,
//                           cn_secret);
// }

// int process_welcome (msg_data_: &[u8],
//                      msg_size_: usize,
//                      cn_precom_: &mut [u8])
// {
//     return process_welcome (msg_data_, msg_size_, server_key, cn_secret,
//                             cn_server, cn_cookie, cn_precom_);
// }

// int produce_initiate (data: *mut c_void,
//                       size: usize,
//                       const cn_nonce_: u64,
//                       metadata_plaintext_: &[u8],
//                       const metadata_length_: usize) const
// {
//     return produce_initiate (data, size, cn_nonce_, server_key,
//                              public_key, secret_key, cn_public, cn_secret,
//                              cn_server, cn_cookie, metadata_plaintext_,
//                              metadata_length_);
// }

//  Our public key (C)
// uint8_t public_key[CRYPTO_BOX_PUBLICKEYBYTES];

//  Our secret key (c)
// uint8_t secret_key[CRYPTO_BOX_SECRETKEYBYTES];

//  Our short-Term public key (C')
// uint8_t cn_public[CRYPTO_BOX_PUBLICKEYBYTES];

//  Our short-Term secret key (c')
// uint8_t cn_secret[CRYPTO_BOX_SECRETKEYBYTES];

//  Server's public key (S)
// uint8_t server_key[CRYPTO_BOX_PUBLICKEYBYTES];

//  Server's short-Term public key (S')
// uint8_t cn_server[CRYPTO_BOX_PUBLICKEYBYTES];

//  Cookie received from server
// uint8_t cn_cookie[16 + 80];

//
pub fn is_handshake_command(msg_data_: &[u8], msg_size_: usize, prefix_: &[u8]) -> bool {
    let mut N = prefix_.len();
    msg_size_ >= (N - 1) && msg_data_[N - 1] == prefix_[N - 1]
    // return msg_size_ >= (N - 1) && !memcmp (msg_data_, prefix_, N - 1);
}

// #endif

// #endif
