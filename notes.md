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
  connection is made, at which point the socket enters a 'Ready' state. In the mute state the socket blocks or drops
  messages according to its type. Following a call to 'zmq_connect' the socket enters the 'Ready' state.

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
* following 'zmq_connect' socket types except 'ZMQ_ROUTER' enter a 'Ready' state. 'ZMQ_ROUTER' enters the 'Ready' state
  when Handshaking is completed.
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
* any blocking operations currently in progress on sockets open within 'context' shall return immediately with an Error
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
zmq_msg_init(&mut copy)
zmq_msg_copy(&mut copy, &msg)
zmq_msg_close(&mut msg)
zmq_msg_close(&mut copy)
```

### zmq_msg_data

* get a pointer to the message content

### zmq_msg_get, zmq_msg_gets

* get a message property
* properties:
    * ZMQ_MORE: get indication of whether the message currently being read has more message parts to follow
    * ZMQ_SRCFD: get the pre-allocated socket file descriptor
    * ZMQ_SHARED: get indication of whether the message is a shared reference

```pseudo
msg: ZmqMessage
loop 
    zmq_msg_init(&mut msg)
    zmq_msg_recv(socket, &frame, 0)
    if zmq_msg_get(&msg, ZMQ_MORE) == 0
        break
    zmq_msg_close(&mut msg)
```

### zmq_msg_init, zmq_msg_init_buffer, zmq_msg_init_data, zmq_msg_init_size

* zmq_msg_init: initialize an empty zmq message
* zmq_msg_init_buffer: initalize a message with buffer copy
* zmq_msg_init_data: initialize a message with data copy
* zmq_msg_init_size: initialize a message with size

* initialize the message referenced by 'msg' to an empty message
* the functions zmq_msg_init, zmq_msg_init_data, zmq_msg_init_size, and zmq_msg_init_buffer are mutually exclusive

```pseudo
msg: ZmqMessage
zmq_msg_init(&mut msg)
zmq_msg_recv(socket, &msg, 0)
```

### zmq_msg_more

* indicate if there are more message parts to receive
* indicates if this is part of a multi-part message, and there are further parts to receive.

```pseudo
part: ZmqMessage
loop
    zmq_msg_init(&mut part)
    zmq_msg_recv(socket, &part, 0)
    if zmq_msg_more(&part) == 0
        break
    zmq_msg_close(&mut part)
```

### zmq_msg_move

* move content of a message to another message

### zmq_msg_recv

* receive a message part from a socket
* receive a message part from the socket referenced by the socket argument and store it in the message referenced by
  the msg argument. Any content previously present in the message object referenced by msg is completely reset.
* flags:
    * ZMQ_DONTWAIT: return immediately if there is no message to receive
* if the message is a multi-part message, then each message part is received as a separate ZmqMessage object.

```pseudo
msg: ZmqMessage
zmq_msg_init(&mut msg)
zmq_msg_recv(socket, &msg, 0)
zmq_msg_close(&mut msg)
```

### zmq_msg_routing_id

* return routing ID for message

### zmq_msg_send

* send a message part on a socket
* queue the messag referenced by msg to be sent to the socket referenced by the socket argument. The flags argument
* flags:
    * ZMQ_DONTWAIT: perform operation in non-blocking mode
    * ZMQ_SNDMORE: send multi-part message, more parts to follow
* on success, the message has been queued but not yet sent

```pseudo
rc = zmq_msg_send (&part1, socket, ZMQ_SNDMORE);
rc = zmq_msg_send (&part2, socket, ZMQ_SNDMORE);
rc = zmq_msg_send (&part3, socket, 0);
```

### zmq_msg_set

* set a message property

### zmq_msg_set_routing_id

* set the message's routing id

### zmq_msg_size

* retrieve the message content size

### zmq_poll

* input/output multiplexing
* provides mechanism for multiplexing input/output events in a level-triggered fashion over a set of sockets
* each member of the array pointed to by the items arg is a pollitem structure
* for each pollitem, zmq_poll examines the socket referenced by the socket field or the standard socket specified by the
  file descriptor fd
* if none of the requested events for an item have occurred zmq_pool shall wait for a specified period of time for an
  event to occur.
* flags:
    * ZMQ_POLLIN: at least one message may be received from the socket without blocking
    * ZMQ_POLLOUT: at least one message may be sent to the socket without blocking
    * ZMQ_POLLERR: an Error condition has occurred on the socket
    * ZMQ_POLLPRI: no useful for zmq sockets

```pseudo
items: [ZmqPollItem;2]
items[0].socket = socket1
items[0].events = ZMQ_POLLIN
items[1].socket = None
items[1].fd = fd
items[1].events = ZMQ_POLLIN
zmq_poll(&mut items, 2, -1)
```

### zmq_ppoll

* input/output multiplexing with a signal mask

```pseudo
// simple global signal handler for SIGTERM
static bool sigterm_received = false;
void handle_sigterm (signum: i32) {
    sigterm_received = true;
}

// set up signal mask and install handler for SIGTERM
sigset_t sigmask, sigmask_without_sigterm;
sigemptyset(&sigmask);
sigaddset(&sigmask, SIGTERM);
sigprocmask(SIG_BLOCK, &sigmask, &sigmask_without_sigterm);
struct sigaction sa;
memset(&sa, 0, sizeof(sa));
sa.sa_handler = handle_sigterm;

// poll
zmq_pollitem_t items [1];
// Just one item, which refers to 0MQ socket 'socket' */
items[0].socket = socket;
items[0].events = ZMQ_POLLIN;
// Poll for events indefinitely, but also exit on SIGTERM
int rc = zmq_poll (items, 2, -1, &sigmask_without_sigterm);
if (rc < 0 && errno == EINTR && sigterm_received) {
  // do your SIGTERM business
} else {
  // do your non-SIGTERM Error handling
}
```

### zmq_proxy

* start built-in proxy
* a proxy connects a frontend socket to a backend socket
* data flows from frontend to backend

### zmq_poller_new, zmq_poller_destroy

* manage the lifetime of a poller instance

### zmq_poller_size

* queries the number of sockets or file descriptors registered with a poller

### zmq_poller_add, zmq_poller_modify, zmq_poller_remove

* manages the zmq sockets registered with a poller
* add: registers a socket with a poller
* modify: modifies the subscribed events for a socket
* remove: removes a socket registration completely

### zmq_poller_add_fd, zmq_poller_modify_md, and zmq_poller_remove_fd

* use file descriptors registered with a poller

### zmq_z85_encode

* encode a binary key as Z85 printable text

### zmq_z85_decode

* decode a binary key from Z85 printable text

### zmq_version

* report 0MQ library version

### zmq_unbind

* stop accepting connections on a socket

### zmq_term

** deprecated **

* terminate a 0MQ context

### zmq_socket

* create a 0MQ socket within the specified context
* newly created socket is initially unbound and not assoctiated with any endpoints
* to establish a message flow the socket must first be Connected to at least one endpoint or at least one endpoint must
  be created for accepting connections
* setup, teardown, reconnect, and delivery are transparent to the ZMQ socket interface
* thread safe sockets:
    * ZMQ_CLIENT
    * ZMQ_SERVER
    * ZMQ_DISH
    * ZMQ_RADIO
    * ZMQ_SCATTER
    * ZMQ_GATHER
    * ZMQ_PEER
    * ZMQ_CHANNEL

* socket types:
    * client-server: allow a single ZMQ_SERVER server to one or more ZMQ_CLIENT clients. the client always starts the
      conversation, after which either peer can send messages asynchronouly.
        * ZMQ_CLIENT
        * ZMQ_SERVER
    * radio-dish: one-to-many distrubtion of data from a single publisher to multiple subscribers in a fan-out fashion.
      Radio-dish uses groups versus pub-sub topics. Dish sockets can join a Group. Each message sent by a raddio belongs
      to a Group.
        * ZMQ_RADIO
        * ZMQ_DISH
    * publish-subscribe pattern: one-to-many distribution of data from a single publisher to multiple subscribers in a
      fan out fashion.
        * ZMQ_PUB
        * ZMQ_SUB
        * ZMQ_XPUB, can receive incoming subscription messages from peers.
        * ZMQ_XSUB, can send outgoing subscription messages to peers.
    * pipeline pattern: distributing data to nodes arranged in a pipeline. data always flows down the pipeline and each
      stage is Connected to at least one node. when a stage is Connected to multiple nodes, data is round-robined
      between the Connected nodes.
        * ZMQ_PUSH
        * ZMQ_PULL
    * scatter-gather pattern: thread-safe version of the pipeline pattern
        * ZMQ_SCATTER
        * ZMQ_GATHER
    * exclusive pair pattern: connects two sockets exclusively. only one socket can be Connected to the other. use for
      inter-thread communication across the inproc transport
        * ZMQ_PAIR
    * peer-to-peer pattern: connect a peer to multiple peers. each peer can send and receive messages.Peers can mix and
      match connect and bind on the same socket.
        * ZMQ_PEER
    * channel pattern: thread-safe version of the exclusive-pair pattern
        * ZMQ_CHANNEL
    * native pattern: used for communicating with TCP peers and allows asynchronous requests and replies in either
      direction
        * ZMQ_STREAM: send and receive data from a non-0MQ peer.
    * request-reply pattern: used for sending requests from a ZMQ_REQ client to one or more ZMQ_REP services and
      receiving a subsequent reply for each request sent. each request is round-robined if Connected to multiple
      services. each reply matched with the last issued request.
        * ZMQ_REQ
        * ZMQ_REP
        * ZMQ_DEALER: each message round-robined among all Connected peers.
        * ZMQ_ROUTER: prepend a message part containing the routing id of the originating peer before passing to the
          application. messages are fair-queued from all Connected peers.

```pseudo
void *ctx = zmq_ctx_new ();
assert (ctx);
/* Create ZMQ_STREAM socket */
void *socket = zmq_socket (ctx, ZMQ_STREAM);
assert (socket);
int rc = zmq_bind (socket, "tcp://*:8080");
assert (rc == 0);
/* Data structure to hold the ZMQ_STREAM routing id */
uint8_t routing_id [256];
size_t routing_id_size = 256;
/* Data structure to hold the ZMQ_STREAM received data */
uint8_t raw [256];
size_t raw_size = 256;
while (1) {
	/*  Get HTTP request; routing id frame and then request */
	routing_id_size = zmq_recv (socket, routing_id, 256, 0);
	assert (routing_id_size > 0);
	do {
		raw_size = zmq_recv (socket, raw, 256, 0);
		assert (raw_size >= 0);
	} while (raw_size == 256);
	/* Prepares the response */
	char http_response [] =
		"HTTP/1.0 200 OK\r\n"
		"Content-Type: text/plain\r\n"
		"\r\n"
		"Hello, World!";
	/* Sends the routing id frame followed by the response */
	zmq_send (socket, routing_id, routing_id_size, ZMQ_SNDMORE);
	zmq_send (socket, http_response, strlen (http_response), 0);
	/* Closes the connection by sending the routing id frame followed by a zero response */
	zmq_send (socket, routing_id, routing_id_size, ZMQ_SNDMORE);
	zmq_send (socket, 0, 0, 0);
}
zmq_close (socket);
zmq_ctx_destroy (ctx);
```

### zmq_socket_monitor, zmq_socket_monitor_versioned

* monitor socket events on a socket

### zmq_setsockopt

* set 0MQ socket options
* ZMQ_AFFINITY: set I/O thread affinity
* ZMQ_BACKLOG: set maximum length of the queue of outstanding connections
* ZMQ_BINDTODEVICE: set the network interface for multicast
* ZMQ_BUSY_POLL: removes delays caused by the interrupt and the resultant context switch
* ZMQ_CONNECT_RID: assign the next outbound connection id
* ZMQ_CONNECT_ROUTING_ID: assign the next outbound routing id
* ZMQ_CONFLATE: keep only last message
* ZMQ_CONNECT_TIMEOUT: set connect() timeout
* ZMQ_CURVE_PUBLICKEY: set CURVE public key
* ZMQ_CURVE_SECRETKEY: set CURVE secret key
* ZMQ_CURVE_SERVER: set CURVE server role
* ZMQ_CURVE_SERVERKEY: set CURVE server key
* ZMQ_DISCONNECT_MSG: set a disconnect message that the socket will generate when accepted after peer disconnect
* ZMQ_HICCUP_MSG: set a hiccup msg that the socket will generate when Connected peer temporarily disconnects
* ZMQ_GSSAPI_PLAINTEXT: disable GSSAPI encryption
* ZMQ_GSSAPI_PRINCIPAL: set GSSAPI principal
* ZMQ_GSSAPI_SERVER: set GSSAPI server role
* ZMQ_GSSAPI_SERVICE_PRINCIPAL: set GSSAPI service principal
* ZMQ_GSSAPI_SERVICE_PRINCIPAL_NAMETYPE: set GSSAPI service principal name type
* ZMQ_GSSAPI_PRINCIPAL_NAMETYPE: set the name type of the GSSAPI principal
* ZMQ_HANDSHAKE_IVL: set maximum handshake interval
* ZMQ_HELLO_MSG: set a hello message that will be sent when a new peer connects
* ZMQ_HEARTBEAT_IVL: set the interval between sending ZMTP heartbeats
* ZMQ_HEARTBEAT_TIMEOUT: set the timeout for ZMTP heartbeats
* ZMQ_HEARTBEAT_TTL: set the time-to-live for ZMTP heartbeats
* ZMQ_IDENTITY: set socket identity
* ZMQ_IMMEDIATE: queue messages only to completed connections
* ZMQ_INVERT_MATCHING: invert subscription matching
* ZMQ_IPV6: enable IPv6 on socket
* ZMQ_LINGER: set linger period for socket shutdown
* ZMQ_MAXMSGSIZE: set maximum acceptable inbound message size
* ZMQ_METADATA: add application metadata properties to a socket
* ZMQ_MULTICAST_HOPS: set outbound multicast hop limit
* ZMQ_MULTICAST_MAXTPDU: set maximum outbound multicast datagram size
* ZMQ_PLAIN_PASSWORD: set PLAIN password
* ZMQ_PLAIN_SERVER: set PLAIN server role
* ZMQ_PLAIN_USERNAME: set PLAIN username
* ZMQ_USE_FD: set the pre-allocated socket file descriptor
* ZMQ_PRIORITY: set the priority for the specified socket type
* ZMQ_PROBE_ROUTER: bootstrap connections to ROUTER sockets
* ZMQ_RATE: set multicast data rate
* ZMQ_RCVBUF: set kernel receive buffer size
* ZMQ_RCVHWM: set high water mark for inbound messages
* ZMQ_RCVTIMEO: set timeout for receive operation on socket
* ZMQ_RECONNECT_IVL: set reconnection interval
* ZMQ_RECONNECT_IVL_MAX: set maximum reconnection interval
* ZMQ_RECONNECT_STOP: set condition where reconnect will stop
* ZMQ_RECOVERY_IVL: set multicast recovery interval
* ZMQ_REQ_CORRELATE: match replies with requests
* ZMQ_REQ_RELAXED: relax strict alternation between request and reply
* ZMQ_ROUTER_HANDOVER: handle duplicate client routing ids on ROUTER sockets
* ZMQ_ROUTER_MANDATORY: accept only routable messages on ROUTER sockets
* ZMQ_ROUTER_RAW: switch ROUTER socket to raw mode
* ZMQ_ROUTING_ID: set socket routing ind
* ZMQ_SNDBUF: set kernel transmit buffer size
* ZMQ_SNDHWM: set high water mark for outbound messages
* ZMQ_SNDTIMEO: set timeout for send operation on socket
* ZMQ_SOCKS_PROXY: set SOCKS proxy address
* ZMQ_SOCKS_USERNAME: set SOCKS username and select basic authentication
* ZMQ_SOCKS_PASSWORD: set SOCKS basic authentication password
* ZMQ_STREAM_NOTIFY: enable connect/disconnect events on a TCP listener
* ZMQ_SUBSCRIBE: establish message filter
* ZMQ_TCP_KEEPALIVE: override SO_KEEPALIVE socket option
* ZMQ_TCP_KEEPALIVE_CNT: override TCP_KEEPCNT socket option
* ZMQ_TCP_KEEPALIVE_IDLE: override TCP_KEEPIDLE socket option
* ZMQ_TCP_KEEPALIVE_INTVL: override TCP_KEEPINTVL socket option
* ZMQ_TCP_MAXRT: set TCP Maximum Retransmit Timeout
* ZMQ_TOS: set IP type-of-service for socket
* ZMQ_UNSUBSCRIBE: remove message filter
* ZMQ_XPUB_VERBOSE: provide all subscription messages on XPUB sockets
* ZMQ_XPUB_VERBOSER: provide all subscription messages on XPUB sockets
* ZMQ_XPUB_MANUAL: change the subscription handling to manual
* ZMQ_XPUB_MANUAL_LAST_VALUE: change the subscription handling to manual
* ZMQ_XPUB_NODROP: do not silently drop messages if SENDHWM is reached
* ZMQ_XPUB_WELCOME_MSG: set a welcome message that will be sent to each new subscriber
* ZMQ_XSUB_VERBOSE_UNSUBSCRIBE: pass duplicate unsubscribe messages to the application
* ZMQ_ONLY_FIRST_SUBSCRIBE: process only first subscribe/unsubscribe in a multipart message
* ZMQ_ZAP_DOMAIN: set domain for ZAP authentication
* ZMQ_ZAP_ENFORCE_DOMAIN: set zap domain handling to strictly adhere to the RFC
* ZMQ_TCP_ACCEPT_FILTER: assign filters to allow new TCP connections
* ZMQ_IPC_FILTER_GID: assign Group ID filters to allow new IPC connections
* ZMQ_IPC_FILTER_PID: assign process ID filters to allow new IPC connections
* ZMQ_IPC_FILTER_UID: assign user ID filters to allow new IPC connections
* ZMQ_IPV4ONLY: use only IPV4 on the socket
* ZMQ_VMCI_BUFFER_SIZE: set the buffer size for VMCI connections
* ZMQ_VMCI_BUFFER_MIN_SIZE: set the minimum buffer size for VMCI connections
* ZMQ_VMCI_BUFFER_MAX_SIZE: set the maximum buffer size for VMCI connections
* ZMQ_VMCI_CONNECT_TIMEOUT: set the timeout for VMCI connections
* ZMQ_MULTICAST_LOOP: control multicast loopback
* ZMQ_ROUTER_NOTIFY: enable connect/disconnect events on a ROUTER socket
* ZMQ_IN_BATCH_SIZE: maximal receive batch size
* ZMQ_OUT_BATCH_SIZE: maximal send batch size

### zmq_sendmsq, zmq_send_const, zmq_send,

* send message part on a socket
* flags: ZMQ_DONTWAIT, ZMQ_SNDMORE
* ZMQ_DONTWAIT: perform operation in non-blocking mode
* ZMQ_SNDMORE: send multi-part message, more parts to follow

### zmq_recvmsg, zmq_recv

* receive a message part from a socket
* ZMQ_DONTWAIT: perform an operation in a non-blocking manner

### zmq_proxy_steerable

* built-in proxy with control flow

```pseudo
-------
.Creating a shared queue proxy
----
//  Create frontend, backend and control sockets
void *frontend = zmq_socket (context, ZMQ_ROUTER);
assert (frontend);
void *backend = zmq_socket (context, ZMQ_DEALER);
assert (backend);
void *control = zmq_socket (context, ZMQ_SUB);
assert (control);

//  Bind sockets to TCP ports
assert (zmq_bind (frontend, "tcp://*:5555") == 0);
assert (zmq_bind (backend, "tcp://*:5556") == 0);
assert (zmq_connect (control, "tcp://*:5557") == 0);

// Subscribe to the control socket since we have chosen SUB here
assert (zmq_setsockopt (control, ZMQ_SUBSCRIBE, "", 0));

//  Start the queue proxy, which runs until ETERM or "TERMINATE" 
//  received on the control socket
zmq_proxy_steerable (frontend, backend, NULL, control);
----
.Set up a controller in another node, process or whatever
----
void *control = zmq_socket (context, ZMQ_PUB);
assert (control);
assert (zmq_bind (control, "tcp://*:5557") == 0);

// pause the proxy
assert (zmq_send (control, "PAUSE", 5, 0) == 0);

// resume the proxy
assert (zmq_send (control, "RESUME", 6, 0) == 0);

// terminate the proxy
assert (zmq_send (control, "TERMINATE", 9, 0) == 0);

// check statistics
assert (zmq_send (control, "STATISTICS", 10, 0) == 0);
zmq_ZmqMessage stats_msg;

while (1) {
    assert (zmq_msg_init (&stats_msg) == 0);
    assert (zmq_recvmsg (control, &stats_msg, 0) == sizeof (uint64_t));
    assert (rc == sizeof (uint64_t));
    printf ("Stat: %lu\n", *(unsigned long int *)zmq_msg_data (&stats_msg));
    if (!zmq_msg_get (&stats_msg, ZMQ_MORE))
        break;
    assert (zmq_msg_close (&stats_msg) == 0);
}
assert (zmq_msg_close (&stats_msg) == 0);
```

## ZMQ Poller

* input/output multiplexing
* provide a mechanism for application to multiplex input/output events in a level-triggered fashion over a set of
  sockets

## ZMQ Security

### ZMQ Curve crypto

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

### GSSAPI

* Defines a mechanism for secure authentication and confidentiality for communication between a client and a server.
* Uses the Generics Security Service Application Program Interface (GSSAPI) as defined IETF RFC-2743
* To become a server, the app sets the ZMQ_GSSAPI_SERVER option on the socket
* To become a client, that app sets the ZMQ_GSSAPI_CLIENT option on the socket
* On the client or server the app may set the ZMQ_GSSAPI_PRINCIPAL option to specify the principal name to use for
  authentication. If the principal is not specified, the GSSAPI library will use the default principal for the current
  user.
* Encryption (the default) can be disabled by setting the ZMQ_GSSAPI_PLAINTEXT option on the client and the server

### Null

* no security or confidentiality
* the default configuration for ZMQ sockets

### plain

* clear text authentication and confidentiality
* server sets ZMQ_PLAIN_SERVER option
* client sets ZMQ_PLAIN_USERNAME and ZMQ_PLAIN_PASSWORD options

## ZMQ transports

### ZMQ Inproc

* local in-process (inter-thread) communication transport
* passes messages in-mmeory directly between threads sharing a single context
* no I/O threads involved

```pseudo
zmq_bind(socket, "inproc://some_name");
```

### ZMQ IPC

* local inter-process communication transport
* passess messages between local processes using a system-dependent IPC mechanism

```pseudo
zmq_bind(socket, "ipc://some_name");
zmq_connect(socket, "ipc://some_name");
```

### ZMQ PGM

* reliable multicast transport using the PGM protocol
* ZMQ supports both RFC 3208 pgm transport and encapsulated PGM over UDP
* can only be used with PUB and SUB socket types
* pgm and epgm transports are rate limited by default
* pgm transport requires raw socket access

### ZMQ VMCI

* transport over virtual machine communication interface
* passes messages between virtual machines running on the same host, between virtual machine and the host, and within
  virtual machines

```pseudo
//  VMCI port 5555 on all available interfaces
rc = zmq_bind(socket, "vmci://*:5555");
assert (rc == 0);
//  VMCI port 5555 on the local loop-back interface on all platforms
cid = VMCISock_GetLocalCID();
sprintf(endpoint, "vmci://%d:5555", cid);
rc = zmq_bind(socket, endpoint);
assert (rc == 0);
```

### ZMQ UDP

* UDP multicast and unicast transport
* UDP transport can only be used with ZMQ_RADIO and ZMQ_DISH socket types

```pseudo
//  Unicast - UDP port 5555 on all available interfaces
rc = zmq_bind(dish, "udp://*:5555");
assert (rc == 0);
//  Unicast - UDP port 5555 on the local loop-back interface
rc = zmq_bind(dish, "udp://127.0.0.1:5555");
assert (rc == 0);
//  Unicast - UDP port 5555 on interface eth1
rc = zmq_bind(dish, "udp://eth1:5555");
assert (rc == 0);
//  Multicast - UDP port 5555 on a Multicast address
rc = zmq_bind(dish, "udp://239.0.0.1:5555");
assert (rc == 0);
//  Same as above but joining only on interface eth0
rc = zmq_bind(dish, "udp://eth0;239.0.0.1:5555");
assert (rc == 0);
//  Same as above using IPv6 multicast
rc = zmq_bind(dish, "udp://eth0;[ff02::1]:5555");
assert (rc == 0);

//  Connecting using an Unicast IP address
rc = zmq_connect(radio, "udp://192.168.1.1:5555");
assert (rc == 0);
//  Connecting using a Multicast address
rc = zmq_connect(socket, "udp://239.0.0.1:5555);
assert (rc == 0);
//  Connecting using a Multicast address using local interface wlan0
rc = zmq_connect(socket, "udp://wlan0;239.0.0.1:5555);
assert (rc == 0);
//  Connecting to IPv6 multicast
rc = zmq_connect(socket, "udp://[ff02::1]:5555);
assert (rc == 0);
```

### 0MQ TIPC

* transport over TIPC
* TIPC is a cluster communication protocol with a location transparent addressing scheme

### 0MQ TCP

* unicast transport using TCP
* likely first chice
* the ZMQ HWM system works in concert with the TCP socket buffers at the OS level

```pseudo
//  TCP port 5555 on all available interfaces
rc = zmq_bind(socket, "tcp://*:5555");
assert (rc == 0);
//  TCP port 5555 on the local loop-back interface on all platforms
rc = zmq_bind(socket, "tcp://127.0.0.1:5555");
assert (rc == 0);
//  TCP port 5555 on the first Ethernet network interface on Linux
rc = zmq_bind(socket, "tcp://eth0:5555");
assert (rc == 0);

//  Connecting using an IP address
rc = zmq_connect(socket, "tcp://192.168.1.1:5555");
assert (rc == 0);
//  Connecting using a DNS name
rc = zmq_connect(socket, "tcp://server1:5555");
assert (rc == 0);
//  Connecting using a DNS name and Bind to eth1
rc = zmq_connect(socket, "tcp://eth1:0;server1:5555");
assert (rc == 0);
//  Connecting using a IP address and Bind to an IP address
rc = zmq_connect(socket, "tcp://192.168.1.17:5555;192.168.1.1:5555");
assert (rc == 0);
```
