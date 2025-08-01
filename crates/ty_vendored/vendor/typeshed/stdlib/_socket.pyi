"""Implementation module for socket operations.

See the socket module for documentation.
"""

import sys
from _typeshed import ReadableBuffer, WriteableBuffer
from collections.abc import Iterable
from socket import error as error, gaierror as gaierror, herror as herror, timeout as timeout
from typing import Any, SupportsIndex, overload
from typing_extensions import CapsuleType, TypeAlias

_CMSG: TypeAlias = tuple[int, int, bytes]
_CMSGArg: TypeAlias = tuple[int, int, ReadableBuffer]

# Addresses can be either tuples of varying lengths (AF_INET, AF_INET6,
# AF_NETLINK, AF_TIPC) or strings/buffers (AF_UNIX).
# See getsockaddrarg() in socketmodule.c.
_Address: TypeAlias = tuple[Any, ...] | str | ReadableBuffer
_RetAddress: TypeAlias = Any

# ===== Constants =====
# This matches the order in the CPython documentation
# https://docs.python.org/3/library/socket.html#constants

if sys.platform != "win32":
    AF_UNIX: int

AF_INET: int
AF_INET6: int

AF_UNSPEC: int

SOCK_STREAM: int
SOCK_DGRAM: int
SOCK_RAW: int
SOCK_RDM: int
SOCK_SEQPACKET: int

if sys.platform == "linux":
    # Availability: Linux >= 2.6.27
    SOCK_CLOEXEC: int
    SOCK_NONBLOCK: int

# --------------------
# Many constants of these forms, documented in the Unix documentation on
# sockets and/or the IP protocol, are also defined in the socket module.
# SO_*
# socket.SOMAXCONN
# MSG_*
# SOL_*
# SCM_*
# IPPROTO_*
# IPPORT_*
# INADDR_*
# IP_*
# IPV6_*
# EAI_*
# AI_*
# NI_*
# TCP_*
# --------------------

SO_ACCEPTCONN: int
SO_BROADCAST: int
SO_DEBUG: int
SO_DONTROUTE: int
SO_ERROR: int
SO_KEEPALIVE: int
SO_LINGER: int
SO_OOBINLINE: int
SO_RCVBUF: int
SO_RCVLOWAT: int
SO_RCVTIMEO: int
SO_REUSEADDR: int
SO_SNDBUF: int
SO_SNDLOWAT: int
SO_SNDTIMEO: int
SO_TYPE: int
if sys.platform != "linux":
    SO_USELOOPBACK: int
if sys.platform == "win32":
    SO_EXCLUSIVEADDRUSE: int
if sys.platform != "win32":
    SO_REUSEPORT: int
    if sys.platform != "darwin" or sys.version_info >= (3, 13):
        SO_BINDTODEVICE: int

if sys.platform != "win32" and sys.platform != "darwin":
    SO_DOMAIN: int
    SO_MARK: int
    SO_PASSCRED: int
    SO_PASSSEC: int
    SO_PEERCRED: int
    SO_PEERSEC: int
    SO_PRIORITY: int
    SO_PROTOCOL: int
if sys.platform != "win32" and sys.platform != "darwin" and sys.platform != "linux":
    SO_SETFIB: int
if sys.platform == "linux" and sys.version_info >= (3, 13):
    SO_BINDTOIFINDEX: int

SOMAXCONN: int

MSG_CTRUNC: int
MSG_DONTROUTE: int
MSG_OOB: int
MSG_PEEK: int
MSG_TRUNC: int
MSG_WAITALL: int
if sys.platform != "win32":
    MSG_DONTWAIT: int
    MSG_EOR: int
    MSG_NOSIGNAL: int  # Sometimes this exists on darwin, sometimes not
if sys.platform != "darwin":
    MSG_ERRQUEUE: int
if sys.platform == "win32":
    MSG_BCAST: int
    MSG_MCAST: int
if sys.platform != "win32" and sys.platform != "darwin":
    MSG_CMSG_CLOEXEC: int
    MSG_CONFIRM: int
    MSG_FASTOPEN: int
    MSG_MORE: int
if sys.platform != "win32" and sys.platform != "linux":
    MSG_EOF: int
if sys.platform != "win32" and sys.platform != "linux" and sys.platform != "darwin":
    MSG_NOTIFICATION: int
    MSG_BTAG: int  # Not FreeBSD either
    MSG_ETAG: int  # Not FreeBSD either

SOL_IP: int
SOL_SOCKET: int
SOL_TCP: int
SOL_UDP: int
if sys.platform != "win32" and sys.platform != "darwin":
    # Defined in socket.h for Linux, but these aren't always present for
    # some reason.
    SOL_ATALK: int
    SOL_AX25: int
    SOL_HCI: int
    SOL_IPX: int
    SOL_NETROM: int
    SOL_ROSE: int

if sys.platform != "win32":
    SCM_RIGHTS: int
if sys.platform != "win32" and sys.platform != "darwin":
    SCM_CREDENTIALS: int
if sys.platform != "win32" and sys.platform != "linux":
    SCM_CREDS: int

IPPROTO_ICMP: int
IPPROTO_IP: int
IPPROTO_RAW: int
IPPROTO_TCP: int
IPPROTO_UDP: int
IPPROTO_AH: int
IPPROTO_DSTOPTS: int
IPPROTO_EGP: int
IPPROTO_ESP: int
IPPROTO_FRAGMENT: int
IPPROTO_HOPOPTS: int
IPPROTO_ICMPV6: int
IPPROTO_IDP: int
IPPROTO_IGMP: int
IPPROTO_IPV6: int
IPPROTO_NONE: int
IPPROTO_PIM: int
IPPROTO_PUP: int
IPPROTO_ROUTING: int
IPPROTO_SCTP: int
if sys.platform != "linux":
    IPPROTO_GGP: int
    IPPROTO_IPV4: int
    IPPROTO_MAX: int
    IPPROTO_ND: int
if sys.platform == "win32":
    IPPROTO_CBT: int
    IPPROTO_ICLFXBM: int
    IPPROTO_IGP: int
    IPPROTO_L2TP: int
    IPPROTO_PGM: int
    IPPROTO_RDP: int
    IPPROTO_ST: int
if sys.platform != "win32":
    IPPROTO_GRE: int
    IPPROTO_IPIP: int
    IPPROTO_RSVP: int
    IPPROTO_TP: int
if sys.platform != "win32" and sys.platform != "linux":
    IPPROTO_EON: int
    IPPROTO_HELLO: int
    IPPROTO_IPCOMP: int
    IPPROTO_XTP: int
if sys.platform != "win32" and sys.platform != "darwin" and sys.platform != "linux":
    IPPROTO_BIP: int  # Not FreeBSD either
    IPPROTO_MOBILE: int  # Not FreeBSD either
    IPPROTO_VRRP: int  # Not FreeBSD either
if sys.platform == "linux":
    # Availability: Linux >= 2.6.20, FreeBSD >= 10.1
    IPPROTO_UDPLITE: int
if sys.version_info >= (3, 10) and sys.platform == "linux":
    IPPROTO_MPTCP: int

IPPORT_RESERVED: int
IPPORT_USERRESERVED: int

INADDR_ALLHOSTS_GROUP: int
INADDR_ANY: int
INADDR_BROADCAST: int
INADDR_LOOPBACK: int
INADDR_MAX_LOCAL_GROUP: int
INADDR_NONE: int
INADDR_UNSPEC_GROUP: int

IP_ADD_MEMBERSHIP: int
IP_DROP_MEMBERSHIP: int
IP_HDRINCL: int
IP_MULTICAST_IF: int
IP_MULTICAST_LOOP: int
IP_MULTICAST_TTL: int
IP_OPTIONS: int
if sys.platform != "linux":
    IP_RECVDSTADDR: int
if sys.version_info >= (3, 10):
    IP_RECVTOS: int
IP_TOS: int
IP_TTL: int
if sys.platform != "win32":
    IP_DEFAULT_MULTICAST_LOOP: int
    IP_DEFAULT_MULTICAST_TTL: int
    IP_MAX_MEMBERSHIPS: int
    IP_RECVOPTS: int
    IP_RECVRETOPTS: int
    IP_RETOPTS: int
if sys.version_info >= (3, 13) and sys.platform == "linux":
    CAN_RAW_ERR_FILTER: int
if sys.version_info >= (3, 14):
    IP_RECVTTL: int

    if sys.platform == "win32" or sys.platform == "linux":
        IPV6_RECVERR: int
        IP_RECVERR: int
        SO_ORIGINAL_DST: int

    if sys.platform == "win32":
        SOL_RFCOMM: int
        SO_BTH_ENCRYPT: int
        SO_BTH_MTU: int
        SO_BTH_MTU_MAX: int
        SO_BTH_MTU_MIN: int
        TCP_QUICKACK: int

    if sys.platform == "linux":
        IP_FREEBIND: int
        IP_RECVORIGDSTADDR: int
        VMADDR_CID_LOCAL: int

if sys.platform != "win32" and sys.platform != "darwin":
    IP_TRANSPARENT: int
if sys.platform != "win32" and sys.platform != "darwin" and sys.version_info >= (3, 11):
    IP_BIND_ADDRESS_NO_PORT: int
if sys.version_info >= (3, 12):
    IP_ADD_SOURCE_MEMBERSHIP: int
    IP_BLOCK_SOURCE: int
    IP_DROP_SOURCE_MEMBERSHIP: int
    IP_PKTINFO: int
    IP_UNBLOCK_SOURCE: int

IPV6_CHECKSUM: int
IPV6_JOIN_GROUP: int
IPV6_LEAVE_GROUP: int
IPV6_MULTICAST_HOPS: int
IPV6_MULTICAST_IF: int
IPV6_MULTICAST_LOOP: int
IPV6_RECVTCLASS: int
IPV6_TCLASS: int
IPV6_UNICAST_HOPS: int
IPV6_V6ONLY: int
IPV6_DONTFRAG: int
IPV6_HOPLIMIT: int
IPV6_HOPOPTS: int
IPV6_PKTINFO: int
IPV6_RECVRTHDR: int
IPV6_RTHDR: int
if sys.platform != "win32":
    IPV6_RTHDR_TYPE_0: int
    IPV6_DSTOPTS: int
    IPV6_NEXTHOP: int
    IPV6_PATHMTU: int
    IPV6_RECVDSTOPTS: int
    IPV6_RECVHOPLIMIT: int
    IPV6_RECVHOPOPTS: int
    IPV6_RECVPATHMTU: int
    IPV6_RECVPKTINFO: int
    IPV6_RTHDRDSTOPTS: int

if sys.platform != "win32" and sys.platform != "linux":
    IPV6_USE_MIN_MTU: int

EAI_AGAIN: int
EAI_BADFLAGS: int
EAI_FAIL: int
EAI_FAMILY: int
EAI_MEMORY: int
EAI_NODATA: int
EAI_NONAME: int
EAI_SERVICE: int
EAI_SOCKTYPE: int
if sys.platform != "win32":
    EAI_ADDRFAMILY: int
    EAI_OVERFLOW: int
    EAI_SYSTEM: int
if sys.platform != "win32" and sys.platform != "linux":
    EAI_BADHINTS: int
    EAI_MAX: int
    EAI_PROTOCOL: int

AI_ADDRCONFIG: int
AI_ALL: int
AI_CANONNAME: int
AI_NUMERICHOST: int
AI_NUMERICSERV: int
AI_PASSIVE: int
AI_V4MAPPED: int
if sys.platform != "win32" and sys.platform != "linux":
    AI_DEFAULT: int
    AI_MASK: int
    AI_V4MAPPED_CFG: int

NI_DGRAM: int
NI_MAXHOST: int
NI_MAXSERV: int
NI_NAMEREQD: int
NI_NOFQDN: int
NI_NUMERICHOST: int
NI_NUMERICSERV: int
if sys.platform == "linux" and sys.version_info >= (3, 13):
    NI_IDN: int

TCP_FASTOPEN: int
TCP_KEEPCNT: int
TCP_KEEPINTVL: int
TCP_MAXSEG: int
TCP_NODELAY: int
if sys.platform != "win32":
    TCP_NOTSENT_LOWAT: int
if sys.platform != "darwin":
    TCP_KEEPIDLE: int
if sys.version_info >= (3, 10) and sys.platform == "darwin":
    TCP_KEEPALIVE: int
if sys.version_info >= (3, 11) and sys.platform == "darwin":
    TCP_CONNECTION_INFO: int

if sys.platform != "win32" and sys.platform != "darwin":
    TCP_CONGESTION: int
    TCP_CORK: int
    TCP_DEFER_ACCEPT: int
    TCP_INFO: int
    TCP_LINGER2: int
    TCP_QUICKACK: int
    TCP_SYNCNT: int
    TCP_USER_TIMEOUT: int
    TCP_WINDOW_CLAMP: int
if sys.platform == "linux" and sys.version_info >= (3, 12):
    TCP_CC_INFO: int
    TCP_FASTOPEN_CONNECT: int
    TCP_FASTOPEN_KEY: int
    TCP_FASTOPEN_NO_COOKIE: int
    TCP_INQ: int
    TCP_MD5SIG: int
    TCP_MD5SIG_EXT: int
    TCP_QUEUE_SEQ: int
    TCP_REPAIR: int
    TCP_REPAIR_OPTIONS: int
    TCP_REPAIR_QUEUE: int
    TCP_REPAIR_WINDOW: int
    TCP_SAVED_SYN: int
    TCP_SAVE_SYN: int
    TCP_THIN_DUPACK: int
    TCP_THIN_LINEAR_TIMEOUTS: int
    TCP_TIMESTAMP: int
    TCP_TX_DELAY: int
    TCP_ULP: int
    TCP_ZEROCOPY_RECEIVE: int

# --------------------
# Specifically documented constants
# --------------------

if sys.platform == "linux":
    # Availability: Linux >= 2.6.25, NetBSD >= 8
    AF_CAN: int
    PF_CAN: int
    SOL_CAN_BASE: int
    SOL_CAN_RAW: int
    CAN_EFF_FLAG: int
    CAN_EFF_MASK: int
    CAN_ERR_FLAG: int
    CAN_ERR_MASK: int
    CAN_RAW: int
    CAN_RAW_FILTER: int
    CAN_RAW_LOOPBACK: int
    CAN_RAW_RECV_OWN_MSGS: int
    CAN_RTR_FLAG: int
    CAN_SFF_MASK: int
    if sys.version_info < (3, 11):
        CAN_RAW_ERR_FILTER: int

if sys.platform == "linux":
    # Availability: Linux >= 2.6.25
    CAN_BCM: int
    CAN_BCM_TX_SETUP: int
    CAN_BCM_TX_DELETE: int
    CAN_BCM_TX_READ: int
    CAN_BCM_TX_SEND: int
    CAN_BCM_RX_SETUP: int
    CAN_BCM_RX_DELETE: int
    CAN_BCM_RX_READ: int
    CAN_BCM_TX_STATUS: int
    CAN_BCM_TX_EXPIRED: int
    CAN_BCM_RX_STATUS: int
    CAN_BCM_RX_TIMEOUT: int
    CAN_BCM_RX_CHANGED: int
    CAN_BCM_SETTIMER: int
    CAN_BCM_STARTTIMER: int
    CAN_BCM_TX_COUNTEVT: int
    CAN_BCM_TX_ANNOUNCE: int
    CAN_BCM_TX_CP_CAN_ID: int
    CAN_BCM_RX_FILTER_ID: int
    CAN_BCM_RX_CHECK_DLC: int
    CAN_BCM_RX_NO_AUTOTIMER: int
    CAN_BCM_RX_ANNOUNCE_RESUME: int
    CAN_BCM_TX_RESET_MULTI_IDX: int
    CAN_BCM_RX_RTR_FRAME: int
    CAN_BCM_CAN_FD_FRAME: int

if sys.platform == "linux":
    # Availability: Linux >= 3.6
    CAN_RAW_FD_FRAMES: int
    # Availability: Linux >= 4.1
    CAN_RAW_JOIN_FILTERS: int
    # Availability: Linux >= 2.6.25
    CAN_ISOTP: int
    # Availability: Linux >= 5.4
    CAN_J1939: int

    J1939_MAX_UNICAST_ADDR: int
    J1939_IDLE_ADDR: int
    J1939_NO_ADDR: int
    J1939_NO_NAME: int
    J1939_PGN_REQUEST: int
    J1939_PGN_ADDRESS_CLAIMED: int
    J1939_PGN_ADDRESS_COMMANDED: int
    J1939_PGN_PDU1_MAX: int
    J1939_PGN_MAX: int
    J1939_NO_PGN: int

    SO_J1939_FILTER: int
    SO_J1939_PROMISC: int
    SO_J1939_SEND_PRIO: int
    SO_J1939_ERRQUEUE: int

    SCM_J1939_DEST_ADDR: int
    SCM_J1939_DEST_NAME: int
    SCM_J1939_PRIO: int
    SCM_J1939_ERRQUEUE: int

    J1939_NLA_PAD: int
    J1939_NLA_BYTES_ACKED: int
    J1939_EE_INFO_NONE: int
    J1939_EE_INFO_TX_ABORT: int
    J1939_FILTER_MAX: int

if sys.version_info >= (3, 12) and sys.platform != "linux" and sys.platform != "win32" and sys.platform != "darwin":
    # Availability: FreeBSD >= 14.0
    AF_DIVERT: int
    PF_DIVERT: int

if sys.platform == "linux":
    # Availability: Linux >= 2.2
    AF_PACKET: int
    PF_PACKET: int
    PACKET_BROADCAST: int
    PACKET_FASTROUTE: int
    PACKET_HOST: int
    PACKET_LOOPBACK: int
    PACKET_MULTICAST: int
    PACKET_OTHERHOST: int
    PACKET_OUTGOING: int

if sys.version_info >= (3, 12) and sys.platform == "linux":
    ETH_P_ALL: int

if sys.platform == "linux":
    # Availability: Linux >= 2.6.30
    AF_RDS: int
    PF_RDS: int
    SOL_RDS: int
    # These are present in include/linux/rds.h but don't always show up
    # here.
    RDS_CANCEL_SENT_TO: int
    RDS_CMSG_RDMA_ARGS: int
    RDS_CMSG_RDMA_DEST: int
    RDS_CMSG_RDMA_MAP: int
    RDS_CMSG_RDMA_STATUS: int
    RDS_CONG_MONITOR: int
    RDS_FREE_MR: int
    RDS_GET_MR: int
    RDS_GET_MR_FOR_DEST: int
    RDS_RDMA_DONTWAIT: int
    RDS_RDMA_FENCE: int
    RDS_RDMA_INVALIDATE: int
    RDS_RDMA_NOTIFY_ME: int
    RDS_RDMA_READWRITE: int
    RDS_RDMA_SILENT: int
    RDS_RDMA_USE_ONCE: int
    RDS_RECVERR: int

    # This is supported by CPython but doesn't seem to be a real thing.
    # The closest existing constant in rds.h is RDS_CMSG_CONG_UPDATE
    # RDS_CMSG_RDMA_UPDATE: int

if sys.platform == "win32":
    SIO_RCVALL: int
    SIO_KEEPALIVE_VALS: int
    SIO_LOOPBACK_FAST_PATH: int
    RCVALL_MAX: int
    RCVALL_OFF: int
    RCVALL_ON: int
    RCVALL_SOCKETLEVELONLY: int

if sys.platform == "linux":
    AF_TIPC: int
    SOL_TIPC: int
    TIPC_ADDR_ID: int
    TIPC_ADDR_NAME: int
    TIPC_ADDR_NAMESEQ: int
    TIPC_CFG_SRV: int
    TIPC_CLUSTER_SCOPE: int
    TIPC_CONN_TIMEOUT: int
    TIPC_CRITICAL_IMPORTANCE: int
    TIPC_DEST_DROPPABLE: int
    TIPC_HIGH_IMPORTANCE: int
    TIPC_IMPORTANCE: int
    TIPC_LOW_IMPORTANCE: int
    TIPC_MEDIUM_IMPORTANCE: int
    TIPC_NODE_SCOPE: int
    TIPC_PUBLISHED: int
    TIPC_SRC_DROPPABLE: int
    TIPC_SUBSCR_TIMEOUT: int
    TIPC_SUB_CANCEL: int
    TIPC_SUB_PORTS: int
    TIPC_SUB_SERVICE: int
    TIPC_TOP_SRV: int
    TIPC_WAIT_FOREVER: int
    TIPC_WITHDRAWN: int
    TIPC_ZONE_SCOPE: int

if sys.platform == "linux":
    # Availability: Linux >= 2.6.38
    AF_ALG: int
    SOL_ALG: int
    ALG_OP_DECRYPT: int
    ALG_OP_ENCRYPT: int
    ALG_OP_SIGN: int
    ALG_OP_VERIFY: int
    ALG_SET_AEAD_ASSOCLEN: int
    ALG_SET_AEAD_AUTHSIZE: int
    ALG_SET_IV: int
    ALG_SET_KEY: int
    ALG_SET_OP: int
    ALG_SET_PUBKEY: int

if sys.platform == "linux":
    # Availability: Linux >= 4.8 (or maybe 3.9, CPython docs are confusing)
    AF_VSOCK: int
    IOCTL_VM_SOCKETS_GET_LOCAL_CID: int
    VMADDR_CID_ANY: int
    VMADDR_CID_HOST: int
    VMADDR_PORT_ANY: int
    SO_VM_SOCKETS_BUFFER_MAX_SIZE: int
    SO_VM_SOCKETS_BUFFER_SIZE: int
    SO_VM_SOCKETS_BUFFER_MIN_SIZE: int
    VM_SOCKETS_INVALID_VERSION: int  # undocumented

# Documented as only available on BSD, macOS, but empirically sometimes
# available on Windows
if sys.platform != "linux":
    AF_LINK: int

has_ipv6: bool

if sys.platform != "darwin" and sys.platform != "linux":
    BDADDR_ANY: str
    BDADDR_LOCAL: str

if sys.platform != "win32" and sys.platform != "darwin" and sys.platform != "linux":
    HCI_FILTER: int  # not in NetBSD or DragonFlyBSD
    HCI_TIME_STAMP: int  # not in FreeBSD, NetBSD, or DragonFlyBSD
    HCI_DATA_DIR: int  # not in FreeBSD, NetBSD, or DragonFlyBSD

if sys.platform == "linux":
    AF_QIPCRTR: int  # Availability: Linux >= 4.7

if sys.version_info >= (3, 11) and sys.platform != "linux" and sys.platform != "win32" and sys.platform != "darwin":
    # FreeBSD
    SCM_CREDS2: int
    LOCAL_CREDS: int
    LOCAL_CREDS_PERSISTENT: int

if sys.version_info >= (3, 11) and sys.platform == "linux":
    SO_INCOMING_CPU: int  # Availability: Linux >= 3.9

if sys.version_info >= (3, 12) and sys.platform == "win32":
    # Availability: Windows
    AF_HYPERV: int
    HV_PROTOCOL_RAW: int
    HVSOCKET_CONNECT_TIMEOUT: int
    HVSOCKET_CONNECT_TIMEOUT_MAX: int
    HVSOCKET_CONNECTED_SUSPEND: int
    HVSOCKET_ADDRESS_FLAG_PASSTHRU: int
    HV_GUID_ZERO: str
    HV_GUID_WILDCARD: str
    HV_GUID_BROADCAST: str
    HV_GUID_CHILDREN: str
    HV_GUID_LOOPBACK: str
    HV_GUID_PARENT: str

if sys.version_info >= (3, 12):
    if sys.platform != "win32":
        # Availability: Linux, FreeBSD, macOS
        ETHERTYPE_ARP: int
        ETHERTYPE_IP: int
        ETHERTYPE_IPV6: int
        ETHERTYPE_VLAN: int

# --------------------
# Semi-documented constants
# These are alluded to under the "Socket families" section in the docs
# https://docs.python.org/3/library/socket.html#socket-families
# --------------------

if sys.platform == "linux":
    # Netlink is defined by Linux
    AF_NETLINK: int
    NETLINK_CRYPTO: int
    NETLINK_DNRTMSG: int
    NETLINK_FIREWALL: int
    NETLINK_IP6_FW: int
    NETLINK_NFLOG: int
    NETLINK_ROUTE: int
    NETLINK_USERSOCK: int
    NETLINK_XFRM: int
    # Technically still supported by CPython
    # NETLINK_ARPD: int  # linux 2.0 to 2.6.12 (EOL August 2005)
    # NETLINK_ROUTE6: int  # linux 2.2 to 2.6.12 (EOL August 2005)
    # NETLINK_SKIP: int  # linux 2.0 to 2.6.12 (EOL August 2005)
    # NETLINK_TAPBASE: int  # linux 2.2 to 2.6.12 (EOL August 2005)
    # NETLINK_TCPDIAG: int  # linux 2.6.0 to 2.6.13 (EOL December 2005)
    # NETLINK_W1: int  # linux 2.6.13 to 2.6.17 (EOL October 2006)

if sys.platform == "darwin":
    PF_SYSTEM: int
    SYSPROTO_CONTROL: int

if sys.platform != "darwin" and sys.platform != "linux":
    AF_BLUETOOTH: int

if sys.platform != "win32" and sys.platform != "darwin" and sys.platform != "linux":
    # Linux and some BSD support is explicit in the docs
    # Windows and macOS do not support in practice
    BTPROTO_HCI: int
    BTPROTO_L2CAP: int
    BTPROTO_SCO: int  # not in FreeBSD
if sys.platform != "darwin" and sys.platform != "linux":
    BTPROTO_RFCOMM: int

if sys.platform == "linux":
    UDPLITE_RECV_CSCOV: int
    UDPLITE_SEND_CSCOV: int

# --------------------
# Documented under socket.shutdown
# --------------------
SHUT_RD: int
SHUT_RDWR: int
SHUT_WR: int

# --------------------
# Undocumented constants
# --------------------

# Undocumented address families
AF_APPLETALK: int
AF_DECnet: int
AF_IPX: int
AF_SNA: int

if sys.platform != "win32":
    AF_ROUTE: int

if sys.platform == "darwin":
    AF_SYSTEM: int

if sys.platform != "darwin":
    AF_IRDA: int

if sys.platform != "win32" and sys.platform != "darwin":
    AF_ASH: int
    AF_ATMPVC: int
    AF_ATMSVC: int
    AF_AX25: int
    AF_BRIDGE: int
    AF_ECONET: int
    AF_KEY: int
    AF_LLC: int
    AF_NETBEUI: int
    AF_NETROM: int
    AF_PPPOX: int
    AF_ROSE: int
    AF_SECURITY: int
    AF_WANPIPE: int
    AF_X25: int

# Miscellaneous undocumented

if sys.platform != "win32" and sys.platform != "linux":
    LOCAL_PEERCRED: int

if sys.platform != "win32" and sys.platform != "darwin":
    # Defined in linux socket.h, but this isn't always present for
    # some reason.
    IPX_TYPE: int

# ===== Classes =====

class socket:
    """socket(family=AF_INET, type=SOCK_STREAM, proto=0) -> socket object
    socket(family=-1, type=-1, proto=-1, fileno=None) -> socket object

    Open a socket of the given type.  The family argument specifies the
    address family; it defaults to AF_INET.  The type argument specifies
    whether this is a stream (SOCK_STREAM, this is the default)
    or datagram (SOCK_DGRAM) socket.  The protocol argument defaults to 0,
    specifying the default protocol.  Keyword arguments are accepted.
    The socket is created as non-inheritable.

    When a fileno is passed in, family, type and proto are auto-detected,
    unless they are explicitly set.

    A socket object represents one endpoint of a network connection.

    Methods of socket objects (keyword arguments not allowed):

    _accept() -- accept connection, returning new socket fd and client address
    bind(addr) -- bind the socket to a local address
    close() -- close the socket
    connect(addr) -- connect the socket to a remote address
    connect_ex(addr) -- connect, return an error code instead of an exception
    dup() -- return a new socket fd duplicated from fileno()
    fileno() -- return underlying file descriptor
    getpeername() -- return remote address [*]
    getsockname() -- return local address
    getsockopt(level, optname[, buflen]) -- get socket options
    gettimeout() -- return timeout or None
    listen([n]) -- start listening for incoming connections
    recv(buflen[, flags]) -- receive data
    recv_into(buffer[, nbytes[, flags]]) -- receive data (into a buffer)
    recvfrom(buflen[, flags]) -- receive data and sender's address
    recvfrom_into(buffer[, nbytes, [, flags])
      -- receive data and sender's address (into a buffer)
    sendall(data[, flags]) -- send all data
    send(data[, flags]) -- send data, may not send all of it
    sendto(data[, flags], addr) -- send data to a given address
    setblocking(bool) -- set or clear the blocking I/O flag
    getblocking() -- return True if socket is blocking, False if non-blocking
    setsockopt(level, optname, value[, optlen]) -- set socket options
    settimeout(None | float) -- set or clear the timeout
    shutdown(how) -- shut down traffic in one or both directions

     [*] not available on all platforms!
    """

    @property
    def family(self) -> int:
        """the socket family"""

    @property
    def type(self) -> int:
        """the socket type"""

    @property
    def proto(self) -> int:
        """the socket protocol"""
    # F811: "Redefinition of unused `timeout`"
    @property
    def timeout(self) -> float | None:  # noqa: F811
        """the socket timeout"""
    if sys.platform == "win32":
        def __init__(
            self, family: int = ..., type: int = ..., proto: int = ..., fileno: SupportsIndex | bytes | None = ...
        ) -> None: ...
    else:
        def __init__(self, family: int = ..., type: int = ..., proto: int = ..., fileno: SupportsIndex | None = ...) -> None: ...

    def bind(self, address: _Address, /) -> None:
        """bind(address)

        Bind the socket to a local address.  For IP sockets, the address is a
        pair (host, port); the host must refer to the local host. For raw packet
        sockets the address is a tuple (ifname, proto [,pkttype [,hatype [,addr]]])
        """

    def close(self) -> None:
        """close()

        Close the socket.  It cannot be used after this call.
        """

    def connect(self, address: _Address, /) -> None:
        """connect(address)

        Connect the socket to a remote address.  For IP sockets, the address
        is a pair (host, port).
        """

    def connect_ex(self, address: _Address, /) -> int:
        """connect_ex(address) -> errno

        This is like connect(address), but returns an error code (the errno value)
        instead of raising an exception when an error occurs.
        """

    def detach(self) -> int:
        """detach()

        Close the socket object without closing the underlying file descriptor.
        The object cannot be used after this call, but the file descriptor
        can be reused for other purposes.  The file descriptor is returned.
        """

    def fileno(self) -> int:
        """fileno() -> integer

        Return the integer file descriptor of the socket.
        """

    def getpeername(self) -> _RetAddress:
        """getpeername() -> address info

        Return the address of the remote endpoint.  For IP sockets, the address
        info is a pair (hostaddr, port).
        """

    def getsockname(self) -> _RetAddress:
        """getsockname() -> address info

        Return the address of the local endpoint. The format depends on the
        address family. For IPv4 sockets, the address info is a pair
        (hostaddr, port). For IPv6 sockets, the address info is a 4-tuple
        (hostaddr, port, flowinfo, scope_id).
        """

    @overload
    def getsockopt(self, level: int, optname: int, /) -> int:
        """getsockopt(level, option[, buffersize]) -> value

        Get a socket option.  See the Unix manual for level and option.
        If a nonzero buffersize argument is given, the return value is a
        string of that length; otherwise it is an integer.
        """

    @overload
    def getsockopt(self, level: int, optname: int, buflen: int, /) -> bytes: ...
    def getblocking(self) -> bool:
        """getblocking()

        Returns True if socket is in blocking mode, or False if it
        is in non-blocking mode.
        """

    def gettimeout(self) -> float | None:
        """gettimeout() -> timeout

        Returns the timeout in seconds (float) associated with socket
        operations. A timeout of None indicates that timeouts on socket
        operations are disabled.
        """
    if sys.platform == "win32":
        def ioctl(self, control: int, option: int | tuple[int, int, int] | bool, /) -> None:
            """ioctl(cmd, option) -> long

            Control the socket with WSAIoctl syscall. Currently supported 'cmd' values are
            SIO_RCVALL:  'option' must be one of the socket.RCVALL_* constants.
            SIO_KEEPALIVE_VALS:  'option' is a tuple of (onoff, timeout, interval).
            SIO_LOOPBACK_FAST_PATH: 'option' is a boolean value, and is disabled by default
            """

    def listen(self, backlog: int = ..., /) -> None:
        """listen([backlog])

        Enable a server to accept connections.  If backlog is specified, it must be
        at least 0 (if it is lower, it is set to 0); it specifies the number of
        unaccepted connections that the system will allow before refusing new
        connections. If not specified, a default reasonable value is chosen.
        """

    def recv(self, bufsize: int, flags: int = ..., /) -> bytes:
        """recv(buffersize[, flags]) -> data

        Receive up to buffersize bytes from the socket.  For the optional flags
        argument, see the Unix manual.  When no data is available, block until
        at least one byte is available or until the remote end is closed.  When
        the remote end is closed and all data is read, return the empty string.
        """

    def recvfrom(self, bufsize: int, flags: int = ..., /) -> tuple[bytes, _RetAddress]:
        """recvfrom(buffersize[, flags]) -> (data, address info)

        Like recv(buffersize, flags) but also return the sender's address info.
        """
    if sys.platform != "win32":
        def recvmsg(self, bufsize: int, ancbufsize: int = ..., flags: int = ..., /) -> tuple[bytes, list[_CMSG], int, Any]:
            """recvmsg(bufsize[, ancbufsize[, flags]]) -> (data, ancdata, msg_flags, address)

            Receive normal data (up to bufsize bytes) and ancillary data from the
            socket.  The ancbufsize argument sets the size in bytes of the
            internal buffer used to receive the ancillary data; it defaults to 0,
            meaning that no ancillary data will be received.  Appropriate buffer
            sizes for ancillary data can be calculated using CMSG_SPACE() or
            CMSG_LEN(), and items which do not fit into the buffer might be
            truncated or discarded.  The flags argument defaults to 0 and has the
            same meaning as for recv().

            The return value is a 4-tuple: (data, ancdata, msg_flags, address).
            The data item is a bytes object holding the non-ancillary data
            received.  The ancdata item is a list of zero or more tuples
            (cmsg_level, cmsg_type, cmsg_data) representing the ancillary data
            (control messages) received: cmsg_level and cmsg_type are integers
            specifying the protocol level and protocol-specific type respectively,
            and cmsg_data is a bytes object holding the associated data.  The
            msg_flags item is the bitwise OR of various flags indicating
            conditions on the received message; see your system documentation for
            details.  If the receiving socket is unconnected, address is the
            address of the sending socket, if available; otherwise, its value is
            unspecified.

            If recvmsg() raises an exception after the system call returns, it
            will first attempt to close any file descriptors received via the
            SCM_RIGHTS mechanism.
            """

        def recvmsg_into(
            self, buffers: Iterable[WriteableBuffer], ancbufsize: int = ..., flags: int = ..., /
        ) -> tuple[int, list[_CMSG], int, Any]:
            """recvmsg_into(buffers[, ancbufsize[, flags]]) -> (nbytes, ancdata, msg_flags, address)

            Receive normal data and ancillary data from the socket, scattering the
            non-ancillary data into a series of buffers.  The buffers argument
            must be an iterable of objects that export writable buffers
            (e.g. bytearray objects); these will be filled with successive chunks
            of the non-ancillary data until it has all been written or there are
            no more buffers.  The ancbufsize argument sets the size in bytes of
            the internal buffer used to receive the ancillary data; it defaults to
            0, meaning that no ancillary data will be received.  Appropriate
            buffer sizes for ancillary data can be calculated using CMSG_SPACE()
            or CMSG_LEN(), and items which do not fit into the buffer might be
            truncated or discarded.  The flags argument defaults to 0 and has the
            same meaning as for recv().

            The return value is a 4-tuple: (nbytes, ancdata, msg_flags, address).
            The nbytes item is the total number of bytes of non-ancillary data
            written into the buffers.  The ancdata item is a list of zero or more
            tuples (cmsg_level, cmsg_type, cmsg_data) representing the ancillary
            data (control messages) received: cmsg_level and cmsg_type are
            integers specifying the protocol level and protocol-specific type
            respectively, and cmsg_data is a bytes object holding the associated
            data.  The msg_flags item is the bitwise OR of various flags
            indicating conditions on the received message; see your system
            documentation for details.  If the receiving socket is unconnected,
            address is the address of the sending socket, if available; otherwise,
            its value is unspecified.

            If recvmsg_into() raises an exception after the system call returns,
            it will first attempt to close any file descriptors received via the
            SCM_RIGHTS mechanism.
            """

    def recvfrom_into(self, buffer: WriteableBuffer, nbytes: int = ..., flags: int = ...) -> tuple[int, _RetAddress]:
        """recvfrom_into(buffer[, nbytes[, flags]]) -> (nbytes, address info)

        Like recv_into(buffer[, nbytes[, flags]]) but also return the sender's address info.
        """

    def recv_into(self, buffer: WriteableBuffer, nbytes: int = ..., flags: int = ...) -> int:
        """recv_into(buffer, [nbytes[, flags]]) -> nbytes_read

        A version of recv() that stores its data into a buffer rather than creating
        a new string.  Receive up to buffersize bytes from the socket.  If buffersize
        is not specified (or 0), receive up to the size available in the given buffer.

        See recv() for documentation about the flags.
        """

    def send(self, data: ReadableBuffer, flags: int = ..., /) -> int:
        """send(data[, flags]) -> count

        Send a data string to the socket.  For the optional flags
        argument, see the Unix manual.  Return the number of bytes
        sent; this may be less than len(data) if the network is busy.
        """

    def sendall(self, data: ReadableBuffer, flags: int = ..., /) -> None:
        """sendall(data[, flags])

        Send a data string to the socket.  For the optional flags
        argument, see the Unix manual.  This calls send() repeatedly
        until all data is sent.  If an error occurs, it's impossible
        to tell how much data has been sent.
        """

    @overload
    def sendto(self, data: ReadableBuffer, address: _Address, /) -> int:
        """sendto(data[, flags], address) -> count

        Like send(data, flags) but allows specifying the destination address.
        For IP sockets, the address is a pair (hostaddr, port).
        """

    @overload
    def sendto(self, data: ReadableBuffer, flags: int, address: _Address, /) -> int: ...
    if sys.platform != "win32":
        def sendmsg(
            self,
            buffers: Iterable[ReadableBuffer],
            ancdata: Iterable[_CMSGArg] = ...,
            flags: int = ...,
            address: _Address | None = ...,
            /,
        ) -> int:
            """sendmsg(buffers[, ancdata[, flags[, address]]]) -> count

            Send normal and ancillary data to the socket, gathering the
            non-ancillary data from a series of buffers and concatenating it into
            a single message.  The buffers argument specifies the non-ancillary
            data as an iterable of bytes-like objects (e.g. bytes objects).
            The ancdata argument specifies the ancillary data (control messages)
            as an iterable of zero or more tuples (cmsg_level, cmsg_type,
            cmsg_data), where cmsg_level and cmsg_type are integers specifying the
            protocol level and protocol-specific type respectively, and cmsg_data
            is a bytes-like object holding the associated data.  The flags
            argument defaults to 0 and has the same meaning as for send().  If
            address is supplied and not None, it sets a destination address for
            the message.  The return value is the number of bytes of non-ancillary
            data sent.
            """
    if sys.platform == "linux":
        def sendmsg_afalg(
            self, msg: Iterable[ReadableBuffer] = ..., *, op: int, iv: Any = ..., assoclen: int = ..., flags: int = ...
        ) -> int:
            """sendmsg_afalg([msg], *, op[, iv[, assoclen[, flags=MSG_MORE]]])

            Set operation mode, IV and length of associated data for an AF_ALG
            operation socket.
            """

    def setblocking(self, flag: bool, /) -> None:
        """setblocking(flag)

        Set the socket to blocking (flag is true) or non-blocking (false).
        setblocking(True) is equivalent to settimeout(None);
        setblocking(False) is equivalent to settimeout(0.0).
        """

    def settimeout(self, value: float | None, /) -> None:
        """settimeout(timeout)

        Set a timeout on socket operations.  'timeout' can be a float,
        giving in seconds, or None.  Setting a timeout of None disables
        the timeout feature and is equivalent to setblocking(1).
        Setting a timeout of zero is the same as setblocking(0).
        """

    @overload
    def setsockopt(self, level: int, optname: int, value: int | ReadableBuffer, /) -> None:
        """setsockopt(level, option, value: int)
        setsockopt(level, option, value: buffer)
        setsockopt(level, option, None, optlen: int)

        Set a socket option.  See the Unix manual for level and option.
        The value argument can either be an integer, a string buffer, or
        None, optlen.
        """

    @overload
    def setsockopt(self, level: int, optname: int, value: None, optlen: int, /) -> None: ...
    if sys.platform == "win32":
        def share(self, process_id: int, /) -> bytes:
            """share(process_id) -> bytes

            Share the socket with another process.  The target process id
            must be provided and the resulting bytes object passed to the target
            process.  There the shared socket can be instantiated by calling
            socket.fromshare().
            """

    def shutdown(self, how: int, /) -> None:
        """shutdown(flag)

        Shut down the reading side of the socket (flag == SHUT_RD), the writing side
        of the socket (flag == SHUT_WR), or both ends (flag == SHUT_RDWR).
        """

SocketType = socket

# ===== Functions =====

def close(fd: SupportsIndex, /) -> None:
    """close(integer) -> None

    Close an integer socket file descriptor.  This is like os.close(), but for
    sockets; on some platforms os.close() won't work for socket file descriptors.
    """

def dup(fd: SupportsIndex, /) -> int:
    """dup(integer) -> integer

    Duplicate an integer socket file descriptor.  This is like os.dup(), but for
    sockets; on some platforms os.dup() won't work for socket file descriptors.
    """

# the 5th tuple item is an address
def getaddrinfo(
    host: bytes | str | None,
    port: bytes | str | int | None,
    family: int = ...,
    type: int = ...,
    proto: int = ...,
    flags: int = ...,
) -> list[tuple[int, int, int, str, tuple[str, int] | tuple[str, int, int, int] | tuple[int, bytes]]]:
    """getaddrinfo(host, port [, family, type, proto, flags])
        -> list of (family, type, proto, canonname, sockaddr)

    Resolve host and port into addrinfo struct.
    """

def gethostbyname(hostname: str, /) -> str:
    """gethostbyname(host) -> address

    Return the IP address (a string of the form '255.255.255.255') for a host.
    """

def gethostbyname_ex(hostname: str, /) -> tuple[str, list[str], list[str]]:
    """gethostbyname_ex(host) -> (name, aliaslist, addresslist)

    Return the true host name, a list of aliases, and a list of IP addresses,
    for a host.  The host argument is a string giving a host name or IP number.
    """

def gethostname() -> str:
    """gethostname() -> string

    Return the current host name.
    """

def gethostbyaddr(ip_address: str, /) -> tuple[str, list[str], list[str]]:
    """gethostbyaddr(host) -> (name, aliaslist, addresslist)

    Return the true host name, a list of aliases, and a list of IP addresses,
    for a host.  The host argument is a string giving a host name or IP number.
    """

def getnameinfo(sockaddr: tuple[str, int] | tuple[str, int, int, int] | tuple[int, bytes], flags: int, /) -> tuple[str, str]:
    """getnameinfo(sockaddr, flags) --> (host, port)

    Get host and port for a sockaddr.
    """

def getprotobyname(protocolname: str, /) -> int:
    """getprotobyname(name) -> integer

    Return the protocol number for the named protocol.  (Rarely used.)
    """

def getservbyname(servicename: str, protocolname: str = ..., /) -> int:
    """getservbyname(servicename[, protocolname]) -> integer

    Return a port number from a service name and protocol name.
    The optional protocol name, if given, should be 'tcp' or 'udp',
    otherwise any protocol will match.
    """

def getservbyport(port: int, protocolname: str = ..., /) -> str:
    """getservbyport(port[, protocolname]) -> string

    Return the service name from a port number and protocol name.
    The optional protocol name, if given, should be 'tcp' or 'udp',
    otherwise any protocol will match.
    """

def ntohl(x: int, /) -> int:  # param & ret val are 32-bit ints
    """Convert a 32-bit unsigned integer from network to host byte order."""

def ntohs(x: int, /) -> int:  # param & ret val are 16-bit ints
    """Convert a 16-bit unsigned integer from network to host byte order."""

def htonl(x: int, /) -> int:  # param & ret val are 32-bit ints
    """Convert a 32-bit unsigned integer from host to network byte order."""

def htons(x: int, /) -> int:  # param & ret val are 16-bit ints
    """Convert a 16-bit unsigned integer from host to network byte order."""

def inet_aton(ip_addr: str, /) -> bytes:  # ret val 4 bytes in length
    """Convert an IP address in string format (123.45.67.89) to the 32-bit packed binary format used in low-level network functions."""

def inet_ntoa(packed_ip: ReadableBuffer, /) -> str:
    """Convert an IP address from 32-bit packed binary format to string format."""

def inet_pton(address_family: int, ip_string: str, /) -> bytes:
    """inet_pton(af, ip) -> packed IP address string

    Convert an IP address from string format to a packed string suitable
    for use with low-level network functions.
    """

def inet_ntop(address_family: int, packed_ip: ReadableBuffer, /) -> str:
    """inet_ntop(af, packed_ip) -> string formatted IP address

    Convert a packed IP address of the given family to string format.
    """

def getdefaulttimeout() -> float | None:
    """getdefaulttimeout() -> timeout

    Returns the default timeout in seconds (float) for new socket objects.
    A value of None indicates that new socket objects have no timeout.
    When the socket module is first imported, the default is None.
    """

# F811: "Redefinition of unused `timeout`"
def setdefaulttimeout(timeout: float | None, /) -> None:  # noqa: F811
    """setdefaulttimeout(timeout)

    Set the default timeout in seconds (float) for new socket objects.
    A value of None indicates that new socket objects have no timeout.
    When the socket module is first imported, the default is None.
    """

if sys.platform != "win32":
    def sethostname(name: str, /) -> None:
        """sethostname(name)

        Sets the hostname to name.
        """

    def CMSG_LEN(length: int, /) -> int:
        """CMSG_LEN(length) -> control message length

        Return the total length, without trailing padding, of an ancillary
        data item with associated data of the given length.  This value can
        often be used as the buffer size for recvmsg() to receive a single
        item of ancillary data, but RFC 3542 requires portable applications to
        use CMSG_SPACE() and thus include space for padding, even when the
        item will be the last in the buffer.  Raises OverflowError if length
        is outside the permissible range of values.
        """

    def CMSG_SPACE(length: int, /) -> int:
        """CMSG_SPACE(length) -> buffer size

        Return the buffer size needed for recvmsg() to receive an ancillary
        data item with associated data of the given length, along with any
        trailing padding.  The buffer space needed to receive multiple items
        is the sum of the CMSG_SPACE() values for their associated data
        lengths.  Raises OverflowError if length is outside the permissible
        range of values.
        """

    def socketpair(family: int = ..., type: int = ..., proto: int = ..., /) -> tuple[socket, socket]:
        """socketpair([family[, type [, proto]]]) -> (socket object, socket object)

        Create a pair of socket objects from the sockets returned by the platform
        socketpair() function.
        The arguments are the same as for socket() except the default family is
        AF_UNIX if defined on the platform; otherwise, the default is AF_INET.
        """

def if_nameindex() -> list[tuple[int, str]]:
    """if_nameindex()

    Returns a list of network interface information (index, name) tuples.
    """

def if_nametoindex(oname: str, /) -> int:
    """Returns the interface index corresponding to the interface name if_name."""

if sys.version_info >= (3, 14):
    def if_indextoname(if_index: int, /) -> str:
        """Returns the interface name corresponding to the interface index if_index."""

else:
    def if_indextoname(index: int, /) -> str:
        """if_indextoname(if_index)

        Returns the interface name corresponding to the interface index if_index.
        """

CAPI: CapsuleType
