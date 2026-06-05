/*
 * Aetheris OS Networking Broker C Interface
 * 
 * This header defines the C interface for communicating with the
 * Aetheris POSIX networking broker service.
 */

#ifndef AETHERIS_NET_BROKER_V1_H
#define AETHERIS_NET_BROKER_V1_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Socket family constants */
#define AETHERIS_AF_INET    2
#define AETHERIS_AF_INET6   10
#define AETHERIS_AF_UNIX    1
#define AETHERIS_AF_PACKET  17

/* Socket type constants */
#define AETHERIS_SOCK_STREAM    1
#define AETHERIS_SOCK_DGRAM     2
#define AETHERIS_SOCK_RAW       3
#define AETHERIS_SOCK_SEQPACKET 5

/* Socket state constants */
#define AETHERIS_SOCKET_STATE_CREATED    0
#define AETHERIS_SOCKET_STATE_BOUND      1
#define AETHERIS_SOCKET_STATE_LISTENING  2
#define AETHERIS_SOCKET_STATE_CONNECTED  3
#define AETHERIS_SOCKET_STATE_CLOSED     4
#define AETHERIS_SOCKET_STATE_ERROR      5

/* Poll event constants */
#define AETHERIS_POLLIN     0x0001
#define AETHERIS_POLLPRI    0x0002
#define AETHERIS_POLLOUT    0x0004
#define AETHERIS_POLLERR    0x0008
#define AETHERIS_POLLHUP    0x0010
#define AETHERIS_POLLNVAL   0x0020

/* Maximum values */
#define AETHERIS_MAX_SOCKET_ID_LEN    64
#define AETHERIS_MAX_PROCESS_CAP_LEN  128
#define AETHERIS_MAX_NAMESPACE_LEN    64
#define AETHERIS_MAX_POLL_SOCKETS     64
#define AETHERIS_MAX_ADDRESSES        16
#define AETHERIS_MAX_DATA_LEN         65536

/* Socket address structure */
typedef struct {
    uint16_t family;
    uint16_t port;
    uint8_t addr[16];  /* IPv4 or IPv6 address */
    uint8_t addrlen;
} aetheris_socket_addr_t;

/* Broker request types */
typedef enum {
    AETHERIS_BROKER_REQUEST_SOCKET = 1,
    AETHERIS_BROKER_REQUEST_BIND,
    AETHERIS_BROKER_REQUEST_LISTEN,
    AETHERIS_BROKER_REQUEST_ACCEPT,
    AETHERIS_BROKER_REQUEST_CONNECT,
    AETHERIS_BROKER_REQUEST_SEND,
    AETHERIS_BROKER_REQUEST_RECV,
    AETHERIS_BROKER_REQUEST_CLOSE,
    AETHERIS_BROKER_REQUEST_GET_SOCK_OPT,
    AETHERIS_BROKER_REQUEST_SET_SOCK_OPT,
    AETHERIS_BROKER_REQUEST_SHUTDOWN,
    AETHERIS_BROKER_REQUEST_GET_ADDR_INFO,
    AETHERIS_BROKER_REQUEST_POLL,
    AETHERIS_BROKER_REQUEST_CREATE_NAMESPACE,
    AETHERIS_BROKER_REQUEST_SET_NAMESPACE,
} aetheris_broker_request_type_t;

/* Broker response types */
typedef enum {
    AETHERIS_BROKER_RESPONSE_SOCKET_CREATED = 1,
    AETHERIS_BROKER_RESPONSE_SOCKET_BOUND,
    AETHERIS_BROKER_RESPONSE_SOCKET_LISTENING,
    AETHERIS_BROKER_RESPONSE_CONNECTION_ACCEPTED,
    AETHERIS_BROKER_RESPONSE_CONNECTED,
    AETHERIS_BROKER_RESPONSE_DATA_SENT,
    AETHERIS_BROKER_RESPONSE_DATA_RECEIVED,
    AETHERIS_BROKER_RESPONSE_SOCKET_CLOSED,
    AETHERIS_BROKER_RESPONSE_SOCKET_OPTION,
    AETHERIS_BROKER_RESPONSE_SOCKET_OPTION_SET,
    AETHERIS_BROKER_RESPONSE_SOCKET_SHUTDOWN,
    AETHERIS_BROKER_RESPONSE_ADDRESS_INFO,
    AETHERIS_BROKER_RESPONSE_POLL_RESULTS,
    AETHERIS_BROKER_RESPONSE_NAMESPACE_CREATED,
    AETHERIS_BROKER_RESPONSE_NAMESPACE_SET,
    AETHERIS_BROKER_RESPONSE_ERROR,
} aetheris_broker_response_type_t;

/* Socket request */
typedef struct {
    int32_t domain;
    int32_t type;
    int32_t protocol;
    char process_cap[AETHERIS_MAX_PROCESS_CAP_LEN];
    char namespace[AETHERIS_MAX_NAMESPACE_LEN];
} aetheris_socket_request_t;

/* Bind request */
typedef struct {
    char socket_id[AETHERIS_MAX_SOCKET_ID_LEN];
    aetheris_socket_addr_t address;
    char process_cap[AETHERIS_MAX_PROCESS_CAP_LEN];
} aetheris_bind_request_t;

/* Listen request */
typedef struct {
    char socket_id[AETHERIS_MAX_SOCKET_ID_LEN];
    int32_t backlog;
    char process_cap[AETHERIS_MAX_PROCESS_CAP_LEN];
} aetheris_listen_request_t;

/* Accept request */
typedef struct {
    char socket_id[AETHERIS_MAX_SOCKET_ID_LEN];
    char process_cap[AETHERIS_MAX_PROCESS_CAP_LEN];
} aetheris_accept_request_t;

/* Connect request */
typedef struct {
    char socket_id[AETHERIS_MAX_SOCKET_ID_LEN];
    aetheris_socket_addr_t address;
    char process_cap[AETHERIS_MAX_PROCESS_CAP_LEN];
} aetheris_connect_request_t;

/* Send request */
typedef struct {
    char socket_id[AETHERIS_MAX_SOCKET_ID_LEN];
    uint8_t *data;
    size_t data_len;
    int32_t flags;
    char process_cap[AETHERIS_MAX_PROCESS_CAP_LEN];
} aetheris_send_request_t;

/* Recv request */
typedef struct {
    char socket_id[AETHERIS_MAX_SOCKET_ID_LEN];
    size_t buffer_size;
    int32_t flags;
    char process_cap[AETHERIS_MAX_PROCESS_CAP_LEN];
} aetheris_recv_request_t;

/* Close request */
typedef struct {
    char socket_id[AETHERIS_MAX_SOCKET_ID_LEN];
    char process_cap[AETHERIS_MAX_PROCESS_CAP_LEN];
} aetheris_close_request_t;

/* Get socket option request */
typedef struct {
    char socket_id[AETHERIS_MAX_SOCKET_ID_LEN];
    int32_t level;
    int32_t optname;
    char process_cap[AETHERIS_MAX_PROCESS_CAP_LEN];
} aetheris_get_sock_opt_request_t;

/* Set socket option request */
typedef struct {
    char socket_id[AETHERIS_MAX_SOCKET_ID_LEN];
    int32_t level;
    int32_t optname;
    uint8_t value[256];
    size_t value_len;
    char process_cap[AETHERIS_MAX_PROCESS_CAP_LEN];
} aetheris_set_sock_opt_request_t;

/* Shutdown request */
typedef struct {
    char socket_id[AETHERIS_MAX_SOCKET_ID_LEN];
    int32_t how;
    char process_cap[AETHERIS_MAX_PROCESS_CAP_LEN];
} aetheris_shutdown_request_t;

/* Address info hints */
typedef struct {
    int32_t ai_family;
    int32_t ai_socktype;
    int32_t ai_protocol;
    int32_t ai_flags;
} aetheris_addr_info_hints_t;

/* Get address info request */
typedef struct {
    char node[256];
    char service[64];
    aetheris_addr_info_hints_t hints;
    char process_cap[AETHERIS_MAX_PROCESS_CAP_LEN];
} aetheris_get_addr_info_request_t;

/* Poll socket request */
typedef struct {
    char socket_id[AETHERIS_MAX_SOCKET_ID_LEN];
    int16_t events;
} aetheris_poll_socket_request_t;

/* Poll request */
typedef struct {
    aetheris_poll_socket_request_t sockets[AETHERIS_MAX_POLL_SOCKETS];
    int32_t socket_count;
    int64_t timeout;
    char process_cap[AETHERIS_MAX_PROCESS_CAP_LEN];
} aetheris_poll_request_t;

/* Create namespace request */
typedef struct {
    char name[64];
    int32_t class;
    char process_cap[AETHERIS_MAX_PROCESS_CAP_LEN];
} aetheris_create_namespace_request_t;

/* Set namespace request */
typedef struct {
    char process_cap[AETHERIS_MAX_PROCESS_CAP_LEN];
    char namespace[AETHERIS_MAX_NAMESPACE_LEN];
} aetheris_set_namespace_request_t;

/* Broker request union */
typedef struct {
    aetheris_broker_request_type_t type;
    union {
        aetheris_socket_request_t socket;
        aetheris_bind_request_t bind;
        aetheris_listen_request_t listen;
        aetheris_accept_request_t accept;
        aetheris_connect_request_t connect;
        aetheris_send_request_t send;
        aetheris_recv_request_t recv;
        aetheris_close_request_t close;
        aetheris_get_sock_opt_request_t get_sock_opt;
        aetheris_set_sock_opt_request_t set_sock_opt;
        aetheris_shutdown_request_t shutdown;
        aetheris_get_addr_info_request_t get_addr_info;
        aetheris_poll_request_t poll;
        aetheris_create_namespace_request_t create_namespace;
        aetheris_set_namespace_request_t set_namespace;
    };
} aetheris_broker_request_t;

/* Socket created response */
typedef struct {
    char socket_id[AETHERIS_MAX_SOCKET_ID_LEN];
    int32_t fd;
} aetheris_socket_created_response_t;

/* Connection accepted response */
typedef struct {
    char socket_id[AETHERIS_MAX_SOCKET_ID_LEN];
    int32_t fd;
    aetheris_socket_addr_t peer_addr;
} aetheris_connection_accepted_response_t;

/* Connected response */
typedef struct {
    aetheris_socket_addr_t peer_addr;
} aetheris_connected_response_t;

/* Data sent response */
typedef struct {
    size_t bytes_sent;
} aetheris_data_sent_response_t;

/* Data received response */
typedef struct {
    uint8_t data[AETHERIS_MAX_DATA_LEN];
    size_t data_len;
    aetheris_socket_addr_t peer_addr;
    bool has_peer_addr;
} aetheris_data_received_response_t;

/* Socket option response */
typedef struct {
    uint8_t value[256];
    size_t value_len;
} aetheris_socket_option_response_t;

/* Address info response */
typedef struct {
    aetheris_socket_addr_t addresses[AETHERIS_MAX_ADDRESSES];
    int32_t address_count;
} aetheris_address_info_response_t;

/* Poll event */
typedef struct {
    char socket_id[AETHERIS_MAX_SOCKET_ID_LEN];
    int16_t events;
    int16_t revents;
} aetheris_poll_event_t;

/* Poll results response */
typedef struct {
    aetheris_poll_event_t events[AETHERIS_MAX_POLL_SOCKETS];
    int32_t event_count;
} aetheris_poll_results_response_t;

/* Namespace created response */
typedef struct {
    char namespace_id[64];
} aetheris_namespace_created_response_t;

/* Error response */
typedef struct {
    int32_t error_code;
    char message[256];
} aetheris_error_response_t;

/* Broker response union */
typedef struct {
    aetheris_broker_response_type_t type;
    union {
        aetheris_socket_created_response_t socket_created;
        aetheris_connection_accepted_response_t connection_accepted;
        aetheris_connected_response_t connected;
        aetheris_data_sent_response_t data_sent;
        aetheris_data_received_response_t data_received;
        aetheris_socket_option_response_t socket_option;
        aetheris_address_info_response_t address_info;
        aetheris_poll_results_response_t poll_results;
        aetheris_namespace_created_response_t namespace_created;
        aetheris_error_response_t error;
    };
} aetheris_broker_response_t;

/* Broker connection handle */
typedef struct aetheris_broker aetheris_broker_t;

/* Broker API functions */
aetheris_broker_t *aetheris_broker_connect(void);
void aetheris_broker_disconnect(aetheris_broker_t *broker);
int aetheris_broker_send_request(aetheris_broker_t *broker, 
                                const aetheris_broker_request_t *request,
                                aetheris_broker_response_t *response);

/* Utility functions */
const char *aetheris_broker_get_version(void);
int aetheris_broker_get_stats(aetheris_broker_t *broker, char *stats_json, size_t stats_len);

#ifdef __cplusplus
}
#endif

#endif /* AETHERIS_NET_BROKER_V1_H */
