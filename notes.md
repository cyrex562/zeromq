# Notes

## APIs

### zmq_bind

* zmq_bind: accept incoming connections on a socket. It binds a socket to a local endpoint and accepts incoming
  connections on that endpoint.

* An endpoint is a string consisting of 'transport://' followed by 'address'. 'transport specifies the underlying
  protocol to use. 'address' specifies the transport-specific address to connect to.

* ZMQ supported transports:
    * 'tcp': unicast trasnport using TCP
    * 'ipc': local inter-process communication transport
    * 'inproc': local in-process (inter-thread) communincation transport
    * 'pgm','epgm': reliable multicast transport using PGM.
    * 'vmci': virtual machine communications interface
    * 'udp': unreliable unicast and multicast using UDP

* Every ZMQ socket type except 'ZMQ_PAIR' and 'ZMQ_CHANNEL' supports one-to-many and many-to-one semantics.

* The 'ipc', 'tcp', 'vmci', and 'udp' transports accept wildcard addresses.

* The address syntax may be different for 'zmq_bind' and 'zmq_connect' especially for the 'tcp', 'pgm', and 'epgm'
  transports

* Following a call to 'zmq_bind' the socket enters a 'mute' state unless or until at least one incoming or outgoing
  connection is made, at which point the socket enters a 'ready' state. In the mute state the socket blocks or drops
  messages according to its type. Following a call to 'zmq_connect' the socket enters the 'ready' state.

```pseudocode
socket = zmq_socket(context, ZMQ_PUB)

zmq_bind(socket, "inproc://my_publisher")

zmq_bind(socket, "tcp://eth0:5555")
```

### zmq_close

* `zmq_close` destroys the socket.
* Any outstanding messages physically received from the network but not yet received by the app with 'zmq_recv' will be
  discarded
* must be called exactly once for each socket
* 'zmq_close' completes asynchronously, not freeing resources immediately

### zmq_connect_peer

* create an outgoing connection from a socket and retun the connection routing id
* connects a 'ZMQ_PEER' socket to an 'endpoint' and then returns the endpoint 'routing_id'
* 'zmq_connect_peer' supports 'tcp', 'ipc', 'inproc', 'ws', and 'wss' transports

```pseudocode
socket = zmq_socket(context, ZMQ_PEER)
routing_id = zmq_connect(socket, "tcp://host:port")
msg: zmq_message
zmq_msg_init_data(&msg, "HELLO", 5, NULL, NULL)
zmq_msg_set_routing_id(&msg, routing_id)
zmq_msg_send(&msg, socket, 0)
zmq_msg_close(&msg)
```

### zmq_connect

* create outgoing connection from socket
* connects the 'socket' to an 'endpoint' and then accepts incoming connections on that endpoint
* connections are performed as needed by ZMQ. A successful call does not mean that a connection was actually established
* following 'zmq_connect' socket types except 'ZMQ_ROUTER' enter a 'ready' state. 'ZMQ_ROUTER' enters the 'ready' state
  when handshaking is completed.
* for some socket types multiple connection calls dont make sense. In that case the call is silently ignored. This
  behavior applies to ZMQ_DEALER, ZMQ_SUB, ZMQ_PUB, and ZMQ_REQ socket types

```pseudocode
socket = zmq_socket(context, ZMQ_SUB)
zmq_connect(socket, "inproc://my_publisher")
zmq_connect(socket, "tcp://address:port")
```

### zmq_ctx_destroy

**deprecated**

* terminate a zmq context
* any blocking operations currently in progress on sockets open within 'context' shall return immediately with an error
  code of ETERM.
* any further operations other than zmq_close on sockets in the context will fail
* after interrupting all blocking calls, zmq_ctx_destroy blocks until all sockets open within context have been closed
  by zmq_close; and, for each socket in the context all messages sent have been transferred or the linger period
  expries.
* this function is deprecated by linkzmq:zmq_ctx_term

### zmq_ctx_get_ext

* get extended context options
* retrieves the option specified by the 'option_name' argument.
* accepts all option names accepted by zmq_ctx_get
* options that make sense to retrieve using zmq_ctx_get_ext include:
    * ZMQ_THREAD_NAME_PREFIX: get name prefix for I/O threads

```pseudocode
context = zmq_ctx_new
prefix = "MyApp"
zmq_ctx_set(context, ZMQ_THREAD_NAME_PREFIX, &prefix)
recvd_prefix = zmq_ctx_get(context, ZMQ_THREAD_NAME_PREFIX)
```

### zmq_ctx_get

* get context options
* return the option specified by the 'option_name' argument
* supported options:
    * ZMQ_IO_THREADS: number of I/O threads
    * ZMQ_MAX_SOCKETS: maximum number of sockets allowed
    * ZMQ_MAX_MSGSZ: get maximum msg size allowed for the context
    * ZMQ_ZERO_COPY_RECV: get message decoding strategy
    * ZMQ_SOCKET_LIMIT: Get largest configurable number of sockets
    * ZMQ_IPV6: ipv6 option
    * ZMQ_BLOCKY: get blocky setting; 1, if the the context will block on terminate, 0, if the block forever on context
      terminate gambit was disabled
    * ZMQ_THREAD_SCHED_POLICY: get scheduling policy for I/O threads
    * ZMQ_THREAD_NAME_PREFIX: get name prefix for I/O threads
    * ZMQ_MESSAGE_SIZE: size of ZmqMessage at runtime

```pseudocode
context = zmq_ctx_new()
zmq_ctx_set(context, ZMQ_MAX_SOCKETS, 256)
max_sockets = zmq_ctx_get(context, ZMQ_MAX_SOCKETS)
```

### zmq_ctx_new

* create a new ZMQ context
* ZMQ context is thread safe and lock-free

### zmq_ctx_set, zmq_ctx_set_ext

* set (extended) context options
* set the option specified by the 'option_name' argument to the value specified by 'option_value'

### zmq_ctx_shutdown

* shutdown a zmq context
* causes blocking operations in progress to return immediately
* this function is optional, but the client code is required to call zmq_ctx_term in order to free resources

### zmq_ctx_term

* terminate a ZMQ context
* destroys the context
* functions similar to zmq_close
* replaces functions zmq_term and zmq_ctx_destroy

### zmq_curve_keypair

* create a new curve keypair
* returns a newly generated pseudorandom keypair consisting of a public key and a private key

```pseudocode
public_key: [char;41]
private_key: [char;41]
zmq_curve_keypair(public_key, secret_key)
```

### zmq_curve_public

* derive the public key from a private key

```pseudo
public_key: [char;41]
secret_key: [char;41]
zmq_curve_keypair(public_key, secret_key)
derived_public: [char;41]
zmq_curve_public(derived_public, secret_key)
```

### zmq_disconnect

* disconnect a socket from an endpoint
* disconnects may occur at a later time
* results in the socket not being able to queue any additional messages for transmission
* if the linger period is non-zero the socket will attempt to transmit discarded messages to the peer for the duration
  of the linger period

```pseudo
socket = zmq_socket(context, ZMQ_SUB)
zmq_connect(socket, "inproc://my_publisher")
zmq_disconnect(socket, "inproc://my_publisher")
```

### zmq_getsockopt

* get socket options
* returns the value for the option specified by the 'option_name' argument for the socket pointed to by the 'socket'
  argument
* available options:
    * ZMQ_AFFINITY: retrieve I/O thread affinity
        * ZMQ_BACKLOG: retrieve maximum length of the queue of outstanding connections
        * ZMQ_BINDTODEVICE: name of device socket is bound to
        * ZMQ_CONNECT_TIMEOUT: get connect timeout
        * ZMQ_CURVE_PUBLICKEY: get the socket's current CURVE public key
        * ZMQ_CURVE_SECRETKEY: get the socket's current CURVE secret key
        * ZMQ_CURVE_SERVERKEY: get the socket's current CURVE server key
        * ZMQ_EVENTS: get event state for the socket
        * ZMQ_FD: get file descriptor associated with the socket
        * ZMQ_GSSAPI_PLAINTEXT: get GSSAPI plaintext mode
        * ZMQ_GSSAPI_PRINCIPAL: get GSSAPI principal
        * ZMQ_GSSAPI_SERVER: get GSSAPI server mode
        * ZMQ_GSSAPI_SERVICE_PRINCIPAL: get GSSAPI service principal
        * ZMQ_GSSAPI_SERVICE_PRINCIPAL_NAMETYPE: get GSSAPI service principal name type
        * ZMQ_GSSAPI_PRINCIPLAL_NAMETYPE: get GSSAPI principal name type
        * ZMQ_HANDSHAKE_IVL: get maximum handshake interval
        * ZMQ_IDENTITY: get socket identity
        * ZMQ_IMMEDIATE: get value of the IMMEDIATE flag
        * ZMQ_INVERT_MATCHING: get value of the INVERT_MATCHING flag
        * ZMQ_IPV4ONLY: get value of the IPV4ONLY flag
        * ZMQ_IPV6: get value of the IPV6 flag
        * ZMQ_LAST_ENDPOINT: get last endpoint bound for TCP and IPC transports
        * ZMQ_LINGER: get linger period for socket shutdown
        * ZMQ_MAXMSGSIZE: get maximum message size
        * ZMQ_MECHANISM: get security mechanism
        * ZMQ_MULTICAST_HOPS: get default multicast hop limit
        * ZMQ_MULTICAST_MAXTPDU: get maximum multicast transport data unit size
        * ZMQ_PLAIN_PASSWORD: get password for PLAIN security mechanism
        * ZMQ_PLAIN_SERVER: get PLAIN server mode
        * ZMQ_PLAIN_USERNAME: get username for PLAIN security mechanism
        * ZMQ_USE_FD: get the pre-allocated socket file descriptor
        * ZMQ_PRIORITY: get socket priority
        * ZMQ_RATE: get multicast data rate
        * ZMQ_RCVBUF: get kernel receive buffer size
        * ZMQ_RCVHWM: get high water mark for inbound messages
        * ZMQ_RCVMORE: get indication of whether the message currently being read has more message parts to follow
        * ZMQ_RCVTIMEO: get timeout for receive operation
        * ZMQ_RECONNECT_IVL: get reconnection interval
        * ZMQ_RECONNECT_IVL_MAX: get maximum reconnection interval
        * ZMQ_RECONNECT_STOP: retrieve condition where reconnection will stop
        * ZMQ_RECOVERY_IVL: get multicast recovery interval
        * ZMQ_ROUTING_ID: get socket routing id
        * ZMQ_SNDBUF: get kernel transmit buffer size
        * ZMQ_SNDHWM: get high water mark for outbound messages
        * ZMQ_SNDTIMEO: maximum time before a socket operation returns with EAGAIN
        * ZMQ_SOCKS_PROXY: get SOCKS proxy address
        * ZMQ_TCP_KEEPALIVE: override SO_KEEPALIVE socket option
        * ZMQ_TCP_KEEPALIVE_CNT: override TCP_KEEPCNT socket option
        * ZMQ_TCP_KEEPALIVE_IDLE: override TCP_KEEPIDLE socket option
        * ZMQ_TCP_KEEPALIVE_INTVL: override TCP_KEEPINTVL socket option
        * ZMQ_TCP_MAXRT: get TCP max retransmit option
        * ZMQ_THREAD_SAFE: get thread safety status of the socket
        * ZMQ_TOS: get IP type-of-service for socket
        * ZMQ_TYPE: get socket type
        * ZMQ_ZAP_DOMAIN: get domain for ZAP authentication
        * ZMQ_ZAP_ENFORCE_DOMAIN: get ZAP authentication domain enforced flag
        * ZMQ_VMCI_BUFFER_SIZE: get VMCI buffer size
        * ZMQ_VMCI_BUFFER_MIN_SIZE: get VMCI minimum buffer size
        * ZMQ_VMCI_BUFFER_MAX_SIZE: get VMCI maximum buffer size
        * ZMQ_VMCI_CONNECT_TIMEOUT: get VMCI connect timeout
        * ZMQ_MULTICAST_LOOP: get multicast loopback
        * ZMQ_ROUTER_NOTIFY: get ROUTER socket notification settings
        * ZMQ_ROUTER_BATCH_SIZE: get ROUTER socket batch size
        * ZMQ_OUT_BATCH_SIZE: get socket batch size for outbound messages
        * ZMQ_TOPICS_COUNT: number of topic subscriptions received

    ```pseudo
    sndhwm_size: usize = 0;
    zmq_getsockopt(socket, ZMQ_SNDHWM, &sndhwm_size, &size_of(sndhwm_size));
    ```

### zmq_has

* check if ZMQ has a particular capability
* capability options include:
    * ipc: the library supports the ipc:// protocol
    * pgm: the library supports the pgm:// protocol
    * tipc: the library supports the tipc:// protocol
    * norm: the library supports the norm:// protocol
    * curve: the library supports the CURVE security mechanism
    * gssapi: the library supports the GSSAPI security mechanism
    * draft_api: the library supports the draft API

### zmq_msg_close

* release a zmq message object
* informat the infrastructure that any resources associated with the message object can be released

### zmq_msg_copy

* copy the contnet of a message to another message
* copy the message referenced by 'src' to the message referenced by 'dest'

```pseudo
msg: ZmqMessage
zmq_msg_init_buffer(&mut, "Hello World", 12)
copy: ZmqMessage



## ZMQ Curve crypto

* provides a mechanism for authentication and confidentiality for ZMQ sockets between a client and a server.
* Intended for use on public netweorks
* Sockets must switch between client/server roles; independent of bind/connect direction. Sockets change roles by
  setting new options. Role changes affect all subsequent zmq_connect and zmq_bind calls
* Application sets the ZMQ_CURVE_SERVER option on the socket to make it a server socket. The app then sets the
  ZMQ_CURVE_SECRETYKEY option to provide the socket with its long-term support key.
* Application sets the ZMQ_CURVE_SERVERKEY option on the socket to provide the socket with the public key of the server
  it is connecting to. The app then sets the ZMQ_CURVE_PUBLICKEY option to provide the socket with its long-term public
  key.
* Keys are represented as 32 bytes of binary data or 40 characters of base 85 data. The base 85 encoding is compatible
  with the Z85 functions.
* Test cases should use the following key pair

client:

```txt
    BB88471D65E2659B30C55A5321CEBB5AAB2B70A398645C26DCA2B2FCB43FC518
    Yne@$w-vo<fVvi]a<NY6T1ed:M$fCG*[IaLV{hID

secret:
    7BB864B489AFA3671FBE69101F94B38972F24816DFB01B51656B3FEC8DFD0888
    D:)Q[IlAW!ahhC2ac:9*A}h:p?([4%wOTJ%JR%cs
```

server:

```txt
    54FCBA24E93249969316FB617C872BB0C1D1FF14800427C594CBFACF1BC2D652
    rq:rM>}U?@Lns47E1%kR.o@n%FcmmsL/@{H8]yf7

secret:
    8E0BDD697628B91D8F245587EE95C5B04D48963F79259877B49CD9063AEAD3B7
    JTKVSB%%)wK0E.X)V>+}o?pNmC{O&4W4b!Ni{Lh6
```

## GSSAPI

* Defines a mechanism for secure authentication and confidentiality for communication between a client and a server. 
* Uses the Generics Security Service Application Program Interface (GSSAPI) as defined IETF RFC-2743
* To become a server, the app sets the ZMQ_GSSAPI_SERVER option on the socket
* To become a client, that app sets the ZMQ_GSSAPI_CLIENT option on the socket
* On the client or server the app may set the ZMQ_GSSAPI_PRINCIPAL option to specify the principal name to use for
  authentication. If the principal is not specified, the GSSAPI library will use the default principal for the current
  user.
* Encryption (the default) can be disabled by setting the ZMQ_GSSAPI_PLAINTEXT option on the client and the server

## ZMQ Inproc

* local in-process (inter-thread) communication transport
* passes messages in-mmeory directly between threads sharing a single context
* no I/O threads involved

```pseudo
zmq_bind(socket, "inproc://some_name");
```

## ZMQ IPC

* local inter-process communication transport
* passess messages between local processes using a system-dependent IPC mechanism

```pseudo
zmq_bind(socket, "ipc://some_name");
zmq_connect(socket, "ipc://some_name");
```
