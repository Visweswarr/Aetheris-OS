/*
 * Aetheris OS C Networking Shim
 * 
 * This file provides C bindings for the POSIX networking subsystem,
 * bridging standard socket calls to the Aetheris networking broker.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <unistd.h>
#include <sys/socket.h>
#include <sys/select.h>
#include <sys/poll.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <netdb.h>
#include <fcntl.h>
#include <signal.h>

#include "aetheris/net_broker_v1.h"
#include "aetheris/pqc_tls_v1.h"

/* Global broker connection */
static aetheris_broker_t *g_broker = NULL;
static int g_broker_initialized = 0;

/* Global TLS manager */
static pqc_tls_manager_t g_tls_manager;
static int g_tls_initialized = 0;

/* Socket state tracking */
typedef struct {
    int fd;
    char *socket_id;
    int domain;
    int type;
    int protocol;
    char *process_cap;
    char *namespace;
    int state;
    int non_blocking;
    struct sockaddr_storage local_addr;
    struct sockaddr_storage peer_addr;
    socklen_t local_addrlen;
    socklen_t peer_addrlen;
    /* TLS support */
    int tls_enabled;
    char *tls_session_id;
    char *client_did;
    char *server_did;
} aetheris_socket_t;

#define MAX_SOCKETS 1024
static aetheris_socket_t g_sockets[MAX_SOCKETS];
static int g_socket_count = 0;

/* Forward declarations */
static aetheris_socket_t *find_socket_by_fd(int fd);
static aetheris_socket_t *allocate_socket(void);
static void free_socket(aetheris_socket_t *sock);
static int init_broker_connection(void);
static int send_broker_request(aetheris_broker_request_t *req, aetheris_broker_response_t *resp);
static int convert_sockaddr_to_broker(const struct sockaddr *addr, socklen_t addrlen, 
                                     aetheris_socket_addr_t *broker_addr);
static int convert_broker_to_sockaddr(const aetheris_socket_addr_t *broker_addr,
                                     struct sockaddr *addr, socklen_t *addrlen);

/* Initialize broker connection */
static int init_broker_connection(void) {
    if (g_broker_initialized) {
        return 0;
    }

    g_broker = aetheris_broker_connect();
    if (!g_broker) {
        errno = ECONNREFUSED;
        return -1;
    }

    g_broker_initialized = 1;
    return 0;
}

/* Initialize TLS manager */
static int init_tls_manager(void) {
    if (g_tls_initialized) {
        return 0;
    }

    int result = pqc_tls_manager_init(&g_tls_manager);
    if (result != PQC_TLS_SUCCESS) {
        errno = EIO;
        return -1;
    }

    g_tls_initialized = 1;
    return 0;
}

/* Find socket by file descriptor */
static aetheris_socket_t *find_socket_by_fd(int fd) {
    for (int i = 0; i < g_socket_count; i++) {
        if (g_sockets[i].fd == fd) {
            return &g_sockets[i];
        }
    }
    return NULL;
}

/* Allocate new socket structure */
static aetheris_socket_t *allocate_socket(void) {
    if (g_socket_count >= MAX_SOCKETS) {
        errno = EMFILE;
        return NULL;
    }

    aetheris_socket_t *sock = &g_sockets[g_socket_count++];
    memset(sock, 0, sizeof(*sock));
    sock->fd = g_socket_count; /* Use index as FD */
    sock->state = AETHERIS_SOCKET_STATE_CREATED;
    return sock;
}

/* Free socket structure */
static void free_socket(aetheris_socket_t *sock) {
    if (sock->socket_id) {
        free(sock->socket_id);
    }
    if (sock->process_cap) {
        free(sock->process_cap);
    }
    if (sock->namespace) {
        free(sock->namespace);
    }
    if (sock->tls_session_id) {
        free(sock->tls_session_id);
    }
    if (sock->client_did) {
        free(sock->client_did);
    }
    if (sock->server_did) {
        free(sock->server_did);
    }
    memset(sock, 0, sizeof(*sock));
}

/* Send broker request and get response */
static int send_broker_request(aetheris_broker_request_t *req, aetheris_broker_response_t *resp) {
    if (!g_broker_initialized && init_broker_connection() < 0) {
        return -1;
    }

    int result = aetheris_broker_send_request(g_broker, req, resp);
    if (result < 0) {
        errno = EIO;
        return -1;
    }

    return 0;
}

/* Convert sockaddr to broker address format */
static int convert_sockaddr_to_broker(const struct sockaddr *addr, socklen_t addrlen,
                                     aetheris_socket_addr_t *broker_addr) {
    if (!addr || !broker_addr) {
        return -1;
    }

    memset(broker_addr, 0, sizeof(*broker_addr));

    if (addr->sa_family == AF_INET) {
        const struct sockaddr_in *sin = (const struct sockaddr_in *)addr;
        broker_addr->family = AETHERIS_AF_INET;
        broker_addr->port = ntohs(sin->sin_port);
        memcpy(broker_addr->addr, &sin->sin_addr, sizeof(sin->sin_addr));
        broker_addr->addrlen = sizeof(sin->sin_addr);
    } else if (addr->sa_family == AF_INET6) {
        const struct sockaddr_in6 *sin6 = (const struct sockaddr_in6 *)addr;
        broker_addr->family = AETHERIS_AF_INET6;
        broker_addr->port = ntohs(sin6->sin6_port);
        memcpy(broker_addr->addr, &sin6->sin6_addr, sizeof(sin6->sin6_addr));
        broker_addr->addrlen = sizeof(sin6->sin6_addr);
    } else {
        errno = EAFNOSUPPORT;
        return -1;
    }

    return 0;
}

/* Convert broker address to sockaddr format */
static int convert_broker_to_sockaddr(const aetheris_socket_addr_t *broker_addr,
                                     struct sockaddr *addr, socklen_t *addrlen) {
    if (!broker_addr || !addr || !addrlen) {
        return -1;
    }

    if (broker_addr->family == AETHERIS_AF_INET) {
        if (*addrlen < sizeof(struct sockaddr_in)) {
            errno = EINVAL;
            return -1;
        }
        struct sockaddr_in *sin = (struct sockaddr_in *)addr;
        sin->sin_family = AF_INET;
        sin->sin_port = htons(broker_addr->port);
        memcpy(&sin->sin_addr, broker_addr->addr, sizeof(sin->sin_addr));
        *addrlen = sizeof(struct sockaddr_in);
    } else if (broker_addr->family == AETHERIS_AF_INET6) {
        if (*addrlen < sizeof(struct sockaddr_in6)) {
            errno = EINVAL;
            return -1;
        }
        struct sockaddr_in6 *sin6 = (struct sockaddr_in6 *)addr;
        sin6->sin6_family = AF_INET6;
        sin6->sin6_port = htons(broker_addr->port);
        memcpy(&sin6->sin6_addr, broker_addr->addr, sizeof(sin6->sin6_addr));
        *addrlen = sizeof(struct sockaddr_in6);
    } else {
        errno = EAFNOSUPPORT;
        return -1;
    }

    return 0;
}

/* Socket system call implementation */
int socket(int domain, int type, int protocol) {
    aetheris_socket_t *sock = allocate_socket();
    if (!sock) {
        return -1;
    }

    sock->domain = domain;
    sock->type = type;
    sock->protocol = protocol;
    sock->process_cap = strdup("default"); /* TODO: Get from capability system */
    sock->namespace = strdup("default");   /* TODO: Get from namespace system */

    /* Convert to broker request */
    aetheris_broker_request_t req;
    memset(&req, 0, sizeof(req));
    req.type = AETHERIS_BROKER_REQUEST_SOCKET;
    req.socket.domain = domain;
    req.socket.type = type;
    req.socket.protocol = protocol;
    strncpy(req.socket.process_cap, sock->process_cap, sizeof(req.socket.process_cap) - 1);
    strncpy(req.socket.namespace, sock->namespace, sizeof(req.socket.namespace) - 1);

    aetheris_broker_response_t resp;
    if (send_broker_request(&req, &resp) < 0) {
        free_socket(sock);
        return -1;
    }

    if (resp.type != AETHERIS_BROKER_RESPONSE_SOCKET_CREATED) {
        errno = resp.error.error_code;
        free_socket(sock);
        return -1;
    }

    sock->socket_id = strdup(resp.socket_created.socket_id);
    sock->state = AETHERIS_SOCKET_STATE_CREATED;

    return sock->fd;
}

/* Bind system call implementation */
int bind(int sockfd, const struct sockaddr *addr, socklen_t addrlen) {
    aetheris_socket_t *sock = find_socket_by_fd(sockfd);
    if (!sock) {
        errno = EBADF;
        return -1;
    }

    aetheris_socket_addr_t broker_addr;
    if (convert_sockaddr_to_broker(addr, addrlen, &broker_addr) < 0) {
        return -1;
    }

    aetheris_broker_request_t req;
    memset(&req, 0, sizeof(req));
    req.type = AETHERIS_BROKER_REQUEST_BIND;
    strncpy(req.bind.socket_id, sock->socket_id, sizeof(req.bind.socket_id) - 1);
    req.bind.address = broker_addr;
    strncpy(req.bind.process_cap, sock->process_cap, sizeof(req.bind.process_cap) - 1);

    aetheris_broker_response_t resp;
    if (send_broker_request(&req, &resp) < 0) {
        return -1;
    }

    if (resp.type != AETHERIS_BROKER_RESPONSE_SOCKET_BOUND) {
        errno = resp.error.error_code;
        return -1;
    }

    sock->state = AETHERIS_SOCKET_STATE_BOUND;
    memcpy(&sock->local_addr, addr, addrlen);
    sock->local_addrlen = addrlen;

    return 0;
}

/* Listen system call implementation */
int listen(int sockfd, int backlog) {
    aetheris_socket_t *sock = find_socket_by_fd(sockfd);
    if (!sock) {
        errno = EBADF;
        return -1;
    }

    aetheris_broker_request_t req;
    memset(&req, 0, sizeof(req));
    req.type = AETHERIS_BROKER_REQUEST_LISTEN;
    strncpy(req.listen.socket_id, sock->socket_id, sizeof(req.listen.socket_id) - 1);
    req.listen.backlog = backlog;
    strncpy(req.listen.process_cap, sock->process_cap, sizeof(req.listen.process_cap) - 1);

    aetheris_broker_response_t resp;
    if (send_broker_request(&req, &resp) < 0) {
        return -1;
    }

    if (resp.type != AETHERIS_BROKER_RESPONSE_SOCKET_LISTENING) {
        errno = resp.error.error_code;
        return -1;
    }

    sock->state = AETHERIS_SOCKET_STATE_LISTENING;
    return 0;
}

/* Accept system call implementation */
int accept(int sockfd, struct sockaddr *addr, socklen_t *addrlen) {
    aetheris_socket_t *sock = find_socket_by_fd(sockfd);
    if (!sock) {
        errno = EBADF;
        return -1;
    }

    aetheris_broker_request_t req;
    memset(&req, 0, sizeof(req));
    req.type = AETHERIS_BROKER_REQUEST_ACCEPT;
    strncpy(req.accept.socket_id, sock->socket_id, sizeof(req.accept.socket_id) - 1);
    strncpy(req.accept.process_cap, sock->process_cap, sizeof(req.accept.process_cap) - 1);

    aetheris_broker_response_t resp;
    if (send_broker_request(&req, &resp) < 0) {
        return -1;
    }

    if (resp.type != AETHERIS_BROKER_RESPONSE_CONNECTION_ACCEPTED) {
        errno = resp.error.error_code;
        return -1;
    }

    /* Create new socket for accepted connection */
    aetheris_socket_t *new_sock = allocate_socket();
    if (!new_sock) {
        return -1;
    }

    new_sock->socket_id = strdup(resp.connection_accepted.socket_id);
    new_sock->process_cap = strdup(sock->process_cap);
    new_sock->namespace = strdup(sock->namespace);
    new_sock->state = AETHERIS_SOCKET_STATE_CONNECTED;

    /* Convert peer address if provided */
    if (addr && addrlen) {
        if (convert_broker_to_sockaddr(&resp.connection_accepted.peer_addr, addr, addrlen) < 0) {
            free_socket(new_sock);
            return -1;
        }
    }

    return new_sock->fd;
}

/* Connect system call implementation */
int connect(int sockfd, const struct sockaddr *addr, socklen_t addrlen) {
    aetheris_socket_t *sock = find_socket_by_fd(sockfd);
    if (!sock) {
        errno = EBADF;
        return -1;
    }

    aetheris_socket_addr_t broker_addr;
    if (convert_sockaddr_to_broker(addr, addrlen, &broker_addr) < 0) {
        return -1;
    }

    aetheris_broker_request_t req;
    memset(&req, 0, sizeof(req));
    req.type = AETHERIS_BROKER_REQUEST_CONNECT;
    strncpy(req.connect.socket_id, sock->socket_id, sizeof(req.connect.socket_id) - 1);
    req.connect.address = broker_addr;
    strncpy(req.connect.process_cap, sock->process_cap, sizeof(req.connect.process_cap) - 1);

    aetheris_broker_response_t resp;
    if (send_broker_request(&req, &resp) < 0) {
        return -1;
    }

    if (resp.type != AETHERIS_BROKER_RESPONSE_CONNECTED) {
        errno = resp.error.error_code;
        return -1;
    }

    sock->state = AETHERIS_SOCKET_STATE_CONNECTED;
    memcpy(&sock->peer_addr, addr, addrlen);
    sock->peer_addrlen = addrlen;

    return 0;
}

/* Send system call implementation */
ssize_t send(int sockfd, const void *buf, size_t len, int flags) {
    aetheris_socket_t *sock = find_socket_by_fd(sockfd);
    if (!sock) {
        errno = EBADF;
        return -1;
    }

    aetheris_broker_request_t req;
    memset(&req, 0, sizeof(req));
    req.type = AETHERIS_BROKER_REQUEST_SEND;
    strncpy(req.send.socket_id, sock->socket_id, sizeof(req.send.socket_id) - 1);
    req.send.data = (uint8_t *)buf;
    req.send.data_len = len;
    req.send.flags = flags;
    strncpy(req.send.process_cap, sock->process_cap, sizeof(req.send.process_cap) - 1);

    aetheris_broker_response_t resp;
    if (send_broker_request(&req, &resp) < 0) {
        return -1;
    }

    if (resp.type != AETHERIS_BROKER_RESPONSE_DATA_SENT) {
        errno = resp.error.error_code;
        return -1;
    }

    return resp.data_sent.bytes_sent;
}

/* Recv system call implementation */
ssize_t recv(int sockfd, void *buf, size_t len, int flags) {
    aetheris_socket_t *sock = find_socket_by_fd(sockfd);
    if (!sock) {
        errno = EBADF;
        return -1;
    }

    aetheris_broker_request_t req;
    memset(&req, 0, sizeof(req));
    req.type = AETHERIS_BROKER_REQUEST_RECV;
    strncpy(req.recv.socket_id, sock->socket_id, sizeof(req.recv.socket_id) - 1);
    req.recv.buffer_size = len;
    req.recv.flags = flags;
    strncpy(req.recv.process_cap, sock->process_cap, sizeof(req.recv.process_cap) - 1);

    aetheris_broker_response_t resp;
    if (send_broker_request(&req, &resp) < 0) {
        return -1;
    }

    if (resp.type != AETHERIS_BROKER_RESPONSE_DATA_RECEIVED) {
        errno = resp.error.error_code;
        return -1;
    }

    size_t copy_len = resp.data_received.data_len;
    if (copy_len > len) {
        copy_len = len;
    }

    memcpy(buf, resp.data_received.data, copy_len);
    return copy_len;
}

/* Close system call implementation */
int close(int fd) {
    aetheris_socket_t *sock = find_socket_by_fd(fd);
    if (!sock) {
        errno = EBADF;
        return -1;
    }

    aetheris_broker_request_t req;
    memset(&req, 0, sizeof(req));
    req.type = AETHERIS_BROKER_REQUEST_CLOSE;
    strncpy(req.close.socket_id, sock->socket_id, sizeof(req.close.socket_id) - 1);
    strncpy(req.close.process_cap, sock->process_cap, sizeof(req.close.process_cap) - 1);

    aetheris_broker_response_t resp;
    if (send_broker_request(&req, &resp) < 0) {
        return -1;
    }

    if (resp.type != AETHERIS_BROKER_RESPONSE_SOCKET_CLOSED) {
        errno = resp.error.error_code;
        return -1;
    }

    free_socket(sock);
    return 0;
}

/* Select system call implementation */
int select(int nfds, fd_set *readfds, fd_set *writefds, fd_set *exceptfds, struct timeval *timeout) {
    /* Convert to broker poll request */
    aetheris_broker_request_t req;
    memset(&req, 0, sizeof(req));
    req.type = AETHERIS_BROKER_REQUEST_POLL;
    req.poll.timeout = timeout ? (timeout->tv_sec * 1000 + timeout->tv_usec / 1000) : -1;
    strncpy(req.poll.process_cap, "default", sizeof(req.poll.process_cap) - 1);

    /* Convert file descriptor sets to poll requests */
    int poll_count = 0;
    for (int fd = 0; fd < nfds && poll_count < AETHERIS_MAX_POLL_SOCKETS; fd++) {
        int events = 0;
        if (readfds && FD_ISSET(fd, readfds)) {
            events |= AETHERIS_POLLIN;
        }
        if (writefds && FD_ISSET(fd, writefds)) {
            events |= AETHERIS_POLLOUT;
        }
        if (exceptfds && FD_ISSET(fd, exceptfds)) {
            events |= AETHERIS_POLLERR;
        }

        if (events != 0) {
            aetheris_socket_t *sock = find_socket_by_fd(fd);
            if (sock) {
                strncpy(req.poll.sockets[poll_count].socket_id, sock->socket_id, 
                       sizeof(req.poll.sockets[poll_count].socket_id) - 1);
                req.poll.sockets[poll_count].events = events;
                poll_count++;
            }
        }
    }
    req.poll.socket_count = poll_count;

    aetheris_broker_response_t resp;
    if (send_broker_request(&req, &resp) < 0) {
        return -1;
    }

    if (resp.type != AETHERIS_BROKER_RESPONSE_POLL_RESULTS) {
        errno = resp.error.error_code;
        return -1;
    }

    /* Clear file descriptor sets */
    if (readfds) FD_ZERO(readfds);
    if (writefds) FD_ZERO(writefds);
    if (exceptfds) FD_ZERO(exceptfds);

    /* Set ready file descriptors */
    int ready_count = 0;
    for (int i = 0; i < resp.poll_results.event_count; i++) {
        const aetheris_poll_event_t *event = &resp.poll_results.events[i];
        
        /* Find socket by ID */
        for (int fd = 0; fd < nfds; fd++) {
            aetheris_socket_t *sock = find_socket_by_fd(fd);
            if (sock && strcmp(sock->socket_id, event->socket_id) == 0) {
                if (event->revents & AETHERIS_POLLIN && readfds) {
                    FD_SET(fd, readfds);
                    ready_count++;
                }
                if (event->revents & AETHERIS_POLLOUT && writefds) {
                    FD_SET(fd, writefds);
                    ready_count++;
                }
                if (event->revents & AETHERIS_POLLERR && exceptfds) {
                    FD_SET(fd, exceptfds);
                    ready_count++;
                }
                break;
            }
        }
    }

    return ready_count;
}

/* Poll system call implementation */
int poll(struct pollfd *fds, nfds_t nfds, int timeout) {
    /* Convert to broker poll request */
    aetheris_broker_request_t req;
    memset(&req, 0, sizeof(req));
    req.type = AETHERIS_BROKER_REQUEST_POLL;
    req.poll.timeout = timeout;
    strncpy(req.poll.process_cap, "default", sizeof(req.poll.process_cap) - 1);

    /* Convert pollfd array to poll requests */
    int poll_count = 0;
    for (nfds_t i = 0; i < nfds && poll_count < AETHERIS_MAX_POLL_SOCKETS; i++) {
        if (fds[i].fd >= 0) {
            aetheris_socket_t *sock = find_socket_by_fd(fds[i].fd);
            if (sock) {
                strncpy(req.poll.sockets[poll_count].socket_id, sock->socket_id,
                       sizeof(req.poll.sockets[poll_count].socket_id) - 1);
                req.poll.sockets[poll_count].events = fds[i].events;
                poll_count++;
            }
        }
    }
    req.poll.socket_count = poll_count;

    aetheris_broker_response_t resp;
    if (send_broker_request(&req, &resp) < 0) {
        return -1;
    }

    if (resp.type != AETHERIS_BROKER_RESPONSE_POLL_RESULTS) {
        errno = resp.error.error_code;
        return -1;
    }

    /* Update pollfd array with results */
    int ready_count = 0;
    for (int i = 0; i < resp.poll_results.event_count; i++) {
        const aetheris_poll_event_t *event = &resp.poll_results.events[i];
        
        /* Find corresponding pollfd */
        for (nfds_t j = 0; j < nfds; j++) {
            aetheris_socket_t *sock = find_socket_by_fd(fds[j].fd);
            if (sock && strcmp(sock->socket_id, event->socket_id) == 0) {
                fds[j].revents = event->revents;
                if (event->revents != 0) {
                    ready_count++;
                }
                break;
            }
        }
    }

    return ready_count;
}

/* Getaddrinfo system call implementation */
int getaddrinfo(const char *node, const char *service, const struct addrinfo *hints, struct addrinfo **res) {
    aetheris_broker_request_t req;
    memset(&req, 0, sizeof(req));
    req.type = AETHERIS_BROKER_REQUEST_GET_ADDR_INFO;
    if (node) {
        strncpy(req.get_addr_info.node, node, sizeof(req.get_addr_info.node) - 1);
    }
    if (service) {
        strncpy(req.get_addr_info.service, service, sizeof(req.get_addr_info.service) - 1);
    }
    if (hints) {
        req.get_addr_info.hints.ai_family = hints->ai_family;
        req.get_addr_info.hints.ai_socktype = hints->ai_socktype;
        req.get_addr_info.hints.ai_protocol = hints->ai_protocol;
        req.get_addr_info.hints.ai_flags = hints->ai_flags;
    }
    strncpy(req.get_addr_info.process_cap, "default", sizeof(req.get_addr_info.process_cap) - 1);

    aetheris_broker_response_t resp;
    if (send_broker_request(&req, &resp) < 0) {
        return EAI_SYSTEM;
    }

    if (resp.type != AETHERIS_BROKER_RESPONSE_ADDRESS_INFO) {
        return EAI_FAIL;
    }

    /* Convert broker addresses to addrinfo structures */
    *res = NULL;
    struct addrinfo *prev = NULL;
    
    for (int i = 0; i < resp.address_info.address_count; i++) {
        struct addrinfo *ai = malloc(sizeof(struct addrinfo));
        if (!ai) {
            freeaddrinfo(*res);
            return EAI_MEMORY;
        }

        memset(ai, 0, sizeof(*ai));
        ai->ai_family = resp.address_info.addresses[i].family;
        ai->ai_socktype = hints ? hints->ai_socktype : 0;
        ai->ai_protocol = hints ? hints->ai_protocol : 0;
        ai->ai_flags = hints ? hints->ai_flags : 0;
        ai->ai_next = NULL;

        /* Allocate and set address */
        if (ai->ai_family == AF_INET) {
            ai->ai_addrlen = sizeof(struct sockaddr_in);
            ai->ai_addr = malloc(ai->ai_addrlen);
            if (!ai->ai_addr) {
                free(ai);
                freeaddrinfo(*res);
                return EAI_MEMORY;
            }
            struct sockaddr_in *sin = (struct sockaddr_in *)ai->ai_addr;
            sin->sin_family = AF_INET;
            sin->sin_port = htons(resp.address_info.addresses[i].port);
            memcpy(&sin->sin_addr, resp.address_info.addresses[i].addr, sizeof(sin->sin_addr));
        } else if (ai->ai_family == AF_INET6) {
            ai->ai_addrlen = sizeof(struct sockaddr_in6);
            ai->ai_addr = malloc(ai->ai_addrlen);
            if (!ai->ai_addr) {
                free(ai);
                freeaddrinfo(*res);
                return EAI_MEMORY;
            }
            struct sockaddr_in6 *sin6 = (struct sockaddr_in6 *)ai->ai_addr;
            sin6->sin6_family = AF_INET6;
            sin6->sin6_port = htons(resp.address_info.addresses[i].port);
            memcpy(&sin6->sin6_addr, resp.address_info.addresses[i].addr, sizeof(sin6->sin6_addr));
        }

        if (prev) {
            prev->ai_next = ai;
        } else {
            *res = ai;
        }
        prev = ai;
    }

    return 0;
}

/* Freeaddrinfo system call implementation */
void freeaddrinfo(struct addrinfo *res) {
    while (res) {
        struct addrinfo *next = res->ai_next;
        if (res->ai_addr) {
            free(res->ai_addr);
        }
        free(res);
        res = next;
    }
}

/* TLS-enabled socket functions */

/* Create TLS-enabled socket */
int tls_socket(int domain, int type, int protocol, const char *client_did, const char *server_did) {
    int sockfd = socket(domain, type, protocol);
    if (sockfd < 0) {
        return -1;
    }

    aetheris_socket_t *sock = find_socket_by_fd(sockfd);
    if (!sock) {
        close(sockfd);
        return -1;
    }

    /* Initialize TLS manager if needed */
    if (!g_tls_initialized && init_tls_manager() < 0) {
        close(sockfd);
        return -1;
    }

    /* Enable TLS for this socket */
    sock->tls_enabled = 1;
    if (client_did) {
        sock->client_did = strdup(client_did);
    }
    if (server_did) {
        sock->server_did = strdup(server_did);
    }

    /* Generate unique TLS session ID */
    char session_id[256];
    snprintf(session_id, sizeof(session_id), "tls_session_%d_%lu", 
             sockfd, (unsigned long)time(NULL));
    sock->tls_session_id = strdup(session_id);

    return sockfd;
}

/* Perform TLS handshake */
int tls_handshake(int sockfd) {
    aetheris_socket_t *sock = find_socket_by_fd(sockfd);
    if (!sock || !sock->tls_enabled) {
        errno = EBADF;
        return -1;
    }

    if (!g_tls_initialized && init_tls_manager() < 0) {
        return -1;
    }

    /* Perform mTLS handshake */
    pqc_tls_handshake_result_t result;
    int ret = pqc_tls_mtls_handshake(&g_tls_manager, sock->tls_session_id,
                                    sock->client_did ? sock->client_did : "default_client",
                                    sock->server_did ? sock->server_did : "default_server",
                                    &result);
    if (ret != PQC_TLS_SUCCESS) {
        errno = EIO;
        return -1;
    }

    if (!result.success) {
        errno = ECONNREFUSED;
        return -1;
    }

    return 0;
}

/* TLS-enabled send */
ssize_t tls_send(int sockfd, const void *buf, size_t len, int flags) {
    aetheris_socket_t *sock = find_socket_by_fd(sockfd);
    if (!sock || !sock->tls_enabled) {
        errno = EBADF;
        return -1;
    }

    if (!g_tls_initialized && init_tls_manager() < 0) {
        return -1;
    }

    /* Encrypt data using TLS session */
    uint8_t *encrypted_data = malloc(len + 1024); /* Extra space for encryption overhead */
    if (!encrypted_data) {
        errno = ENOMEM;
        return -1;
    }

    size_t encrypted_len = len + 1024;
    int ret = pqc_tls_encrypt_data(&g_tls_manager, sock->tls_session_id,
                                  (const uint8_t *)buf, len,
                                  encrypted_data, &encrypted_len);
    if (ret != PQC_TLS_SUCCESS) {
        free(encrypted_data);
        errno = EIO;
        return -1;
    }

    /* Send encrypted data */
    ssize_t bytes_sent = send(sockfd, encrypted_data, encrypted_len, flags);
    free(encrypted_data);

    return bytes_sent;
}

/* TLS-enabled recv */
ssize_t tls_recv(int sockfd, void *buf, size_t len, int flags) {
    aetheris_socket_t *sock = find_socket_by_fd(sockfd);
    if (!sock || !sock->tls_enabled) {
        errno = EBADF;
        return -1;
    }

    if (!g_tls_initialized && init_tls_manager() < 0) {
        return -1;
    }

    /* Receive encrypted data */
    uint8_t *encrypted_data = malloc(len + 1024); /* Extra space for encryption overhead */
    if (!encrypted_data) {
        errno = ENOMEM;
        return -1;
    }

    ssize_t encrypted_bytes = recv(sockfd, encrypted_data, len + 1024, flags);
    if (encrypted_bytes < 0) {
        free(encrypted_data);
        return -1;
    }

    /* Decrypt data using TLS session */
    size_t decrypted_len = len;
    int ret = pqc_tls_decrypt_data(&g_tls_manager, sock->tls_session_id,
                                  encrypted_data, encrypted_bytes,
                                  (uint8_t *)buf, &decrypted_len);
    free(encrypted_data);

    if (ret != PQC_TLS_SUCCESS) {
        errno = EIO;
        return -1;
    }

    return decrypted_len;
}

/* Get TLS session statistics */
int tls_get_stats(int sockfd, pqc_tls_manager_stats_t *stats) {
    aetheris_socket_t *sock = find_socket_by_fd(sockfd);
    if (!sock || !sock->tls_enabled) {
        errno = EBADF;
        return -1;
    }

    if (!g_tls_initialized && init_tls_manager() < 0) {
        return -1;
    }

    int ret = pqc_tls_get_manager_stats(&g_tls_manager, stats);
    if (ret != PQC_TLS_SUCCESS) {
        errno = EIO;
        return -1;
    }

    return 0;
}

/* Rekey TLS session */
int tls_rekey(int sockfd, pqc_algorithm_t new_algorithm) {
    aetheris_socket_t *sock = find_socket_by_fd(sockfd);
    if (!sock || !sock->tls_enabled) {
        errno = EBADF;
        return -1;
    }

    if (!g_tls_initialized && init_tls_manager() < 0) {
        return -1;
    }

    int ret = pqc_tls_rekey_session(&g_tls_manager, sock->tls_session_id, new_algorithm);
    if (ret != PQC_TLS_SUCCESS) {
        errno = EIO;
        return -1;
    }

    return 0;
}

/* Cleanup function */
void aetheris_net_cleanup(void) {
    if (g_broker) {
        aetheris_broker_disconnect(g_broker);
        g_broker = NULL;
    }
    g_broker_initialized = 0;
    
    if (g_tls_initialized) {
        pqc_tls_manager_cleanup(&g_tls_manager);
        g_tls_initialized = 0;
    }
    
    for (int i = 0; i < g_socket_count; i++) {
        free_socket(&g_sockets[i]);
    }
    g_socket_count = 0;
}
