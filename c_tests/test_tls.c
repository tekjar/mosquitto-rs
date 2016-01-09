#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <time.h>
#include <string.h>
#include <getopt.h>
#include <sys/utsname.h>
#include <signal.h>
#include <mosquitto.h>
#include <errno.h>
#include <errno.h>
#include <ctype.h>
#include "uthash.h"
#include "utstring.h"
#include "openssl/ssl.h"

static struct mosquitto *m = NULL;

int main(){

	mosquitto_lib_init();
	printf("LIBMOSQUITTO %d\n", LIBMOSQUITTO_VERSION_NUMBER);

	if ((m = mosquitto_new("rtr", 1, NULL)) == NULL) {
		fprintf(stderr, "Out of memory.\n");
		exit(1);
	}

	int rc = mosquitto_tls_set(m,
			"/home/raviteja/Desktop/certs/ca.crt",		/* cafile */
			NULL,			/* capath */
			"/home/raviteja/Desktop/certs/scooter.crt",		/* certfile */
			"/home/raviteja/Desktop/certs/scooter.key",		/* keyfile */
			NULL			/* pw_callback() */
			);

	if (rc != MOSQ_ERR_SUCCESS) {
		fprintf(stderr, "Cannot set TLS CA: %s (check path names)\n",
				mosquitto_strerror(rc));
		exit(3);
	}
#if 1
	mosquitto_tls_opts_set(m,
			SSL_VERIFY_PEER,
			NULL,			/* tls_version: "tlsv1.2", "tlsv1" */
			NULL			/* ciphers */
			);
	mosquitto_tls_insecure_set(m, 1);
#endif
	if ((rc = mosquitto_connect(m, "localhost", 8884, 20)) != MOSQ_ERR_SUCCESS) {
		fprintf(stderr, "%d: Unable to connect: %s\n", rc,
				mosquitto_strerror(rc));
		perror("");
		exit(2);
	}
	
	//mosquitto_loop_forever(m, -1, 1);
	mosquitto_destroy(m);
	mosquitto_lib_cleanup();
}
