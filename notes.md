# Notes

## APIs

### zmq_bind

* zmq_bind: accept incoming connections on a socket. It binds a socket to a local endpoint and accepts incoming connections on that endpoint.

* An endpoint is a string consisting of 'transport://' followed by 'address'. 'transport specifies the underlying protocol to use. 'address' specifies the transport-specific address to connect to.

* ZMQ supported transports:
  * 'tcp': unicast trasnport using TCP
  * 'ipc': local inter-process communication transport
  * 'inproc': local in-process (inter-thread) communincation transport
  * 'pgm','epgm': reliable multicast transport using PGM.
  * 'vmci': virtual machine communications interface
  * 'udp': unreliable unicast and multicast using UDP

* Every ZMQ socket type except 'ZMQ_PAIR' and 'ZMQ_CHANNEL' supports one-to-many and many-to-one semantics.

* The 'ipc', 'tcp', 'vmci', and 'udp' transports accept wildcard addresses.

* The address syntax may be different for 'zmq_bind' and 'zmq_connect' especially for the 'tcp', 'pgm', and 'epgm' transports

* Following a call to 'zmq_bind' the socket enters a 'mute' state unless or until at least one incoming or outgoing connection is made, at which point the socket enters a 'ready' state. In the mute state the socket blocks or drops messages according to its type. Following a call to 'zmq_connect' the socket enters the 'ready' state.

```pseudocode
socket = zmq_socket(context, ZMQ_PUB)

zmq_bind(socket, "inproc://my_publisher")

zmq_bind(socket, "tcp://eth0:5555")
```

### zmq_close

* `zmq_close` destroys the socket. 
* Any outstanding messages physically received from the network but not yet received by the app with 'zmq_recv' will be discarded
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
* following 'zmq_connect' socket types except 'ZMQ_ROUTER' enter a 'ready' state. 'ZMQ_ROUTER' enters the 'ready' state when handshaking is completed.
* for some socket types multiple connection calls dont make sense. In that case the call is silently ignored.  This behavior applies to ZMQ_DEALER, ZMQ_SUB, ZMQ_PUB, and ZMQ_REQ socket types

```pseudocode
socket = zmq_socket(context, ZMQ_SUB)
zmq_connect(socket, "inproc://my_publisher")
zmq_connect(socket, "tcp://address:port")
```

### zmq_ctx_destroy

**deprecated**

* terminate a zmq context
* any blocking operations currently in progress on sockets open within 'context' shall return immediately with an error code of ETERM.
* any further operations other than zmq_close on sockets in the context will fail
* after interrupting all blocking calls, zmq_ctx_destroy blocks until all sockets open within context have been closed by zmq_close; and, for each socket in the context all messages sent have been transferred or the linger period expries.
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
  * ZMQ_BLOCKY: get blocky setting; 1, if the the context will block on terminate, 0, if the block forever on context terminate gambit was disabled
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