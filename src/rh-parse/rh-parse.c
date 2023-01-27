#define TEST 0

// --- strnlen.
#define _POSIX_C_SOURCE 200809L

#define MAX_INPUT_LENGTH 200

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
// --- isdigit
#include <ctype.h>
#include <math.h>
#include <assert.h>

// --- yyparse etc.
#include "rh-parse.tab.h"

// --- yylex, rh_parse_store/get routines.
#include "rh-parse.h"

#include <fish-util/fish-util.h>

static struct {
    int done;
    int end_of_stream;
    char *input;

    // --- this is only for testing, and it only keeps track of one cdata
    // token at a time.
    struct parse *parse;
} global = {0};

struct parse {
    int num;
    // --- we won't own these.
    const char *command;
    const char *dir;
    const char *cdata;
};

static int int_length (int i) {
    // lexer XX
    assert(i > 0);
    return 1 + (int) log10(i);
}


/* FFI */
static void *host_parse_results;

static
rust_callback__int__void
host_cb_store_num;

static
rust_callback__const_char_star_and_size_t__void
host_cb_store_command;

static
rust_callback__const_char_star_and_size_t__void
host_cb_store_dir;

static
rust_callback__const_char_star_and_size_t__void
host_cb_store_cdata;

/* / */

// --- called from parser.
//
// return 0 or negative to end.
int yylex (void) {
    if (global.done)
        return 0;

    if (global.end_of_stream) {
        global.done = 1;
        return T_END;
    }

    int c;

    // XX
    while (1) {
        c = *global.input;
        if (c == '\0') {
            global.done = 1;
            return T_END;
        }
        if (!isspace(c))
            break;
        ++global.input;
    }

    char *token_start_pos = global.input;
    while (1) {
        c = *global.input;
        if (c == '\0') {
            global.end_of_stream = 1;
            break;
        }
        if (isspace(c))
            break;
        ++global.input;
    }
    size_t token_length = global.input - token_start_pos;
    if (token_length == 0)
        return 0;
    char *token = str(1 + token_length);

    // --- including \0.
    snprintf(token, token_length + 1, "%s", token_start_pos);

    if (!strncmp(token_start_pos, "=", token_length))
        return '=';
    if (f_is_int_strn(token, token_length)) {
        int as_int = atoi(token);
        if (as_int > 0) {
            yylval.intval = atoi(token);
            return T_NUM;
        }
    }
    if (!strncmp(token_start_pos, "t", token_length))
        return T_LS_T;
    if (!strncmp(token_start_pos, "tr", token_length))
        return T_LS_TR;
    if (!strncmp(token_start_pos, "l", token_length))
        return T_LS_L;
    if (!strncmp(token_start_pos, "lr", token_length))
        return T_LS_LR;

    // custom:
    if (!strncmp(token_start_pos, "ag", token_length))
        return T_LS_A1;
    if (!strncmp(token_start_pos, "af", token_length))
        return T_LS_A2;

    if (!strncmp(token_start_pos, "z", token_length))
        return T_LS_Z;

    // --- send malloc'd to parser.
    yylval.strval = token;

    return T_CDATA;
}

/* public */

void rh_parse_register_cb_store_num(rust_callback__int__void cb) {
    host_cb_store_num = cb;
}
void rh_parse_register_cb_store_command(rust_callback__const_char_star_and_size_t__void cb) {
    host_cb_store_command = cb;
}
void rh_parse_register_cb_store_dir(rust_callback__const_char_star_and_size_t__void cb) {
    host_cb_store_dir = cb;
}
void rh_parse_register_cb_store_cdata(rust_callback__const_char_star_and_size_t__void cb) {
    host_cb_store_cdata = cb;
}

void rh_parse_init (void *parse_results) {
    host_parse_results = parse_results;

    global.end_of_stream = 0;
    global.done = 0;

    struct parse *parse = malloc(sizeof(struct parse));
    memset(parse, '\0', sizeof(struct parse));
    global.parse = parse;
}

// --- input comes from rust -- dup it.
void rh_parse_set_input (char *the_input) {
    size_t len = strnlen(the_input, MAX_INPUT_LENGTH);
    assert(len < MAX_INPUT_LENGTH);
    global.input = malloc((len + 1) * sizeof(char));
    // --- including \0.
    snprintf(global.input, len + 1, "%s", the_input);
}

int rh_parse_start () {
    return yyparse ();
}

void rh_parse_parse_store_num (int i) {
    global.parse->num = i;

    if (!host_cb_store_num) {
        iwarn("Host callback store_num not set.");
        return;
    }
    host_cb_store_num(host_parse_results, i);
}

int rh_parse_parse_get_num () {
    return global.parse->num;
}

void rh_parse_parse_store_command (const char *i, size_t len) {
    global.parse->command = i;

    if (!host_cb_store_command) {
        iwarn("Host callback store_command not set.");
        return;
    }
    host_cb_store_command(host_parse_results, i, len);
}

const char *rh_parse_parse_get_command () {
    return global.parse->command;
}

void rh_parse_parse_store_cdata (const char *i, size_t len) {
    global.parse->cdata = i;

    if (!host_cb_store_cdata) {
        iwarn("Host callback store_cdata not set.");
        return;
    }
    host_cb_store_cdata(host_parse_results, i, len);
}

const char *rh_parse_parse_get_cdata () {
    return global.parse->cdata;
}

void rh_parse_parse_store_dir (const char *i, size_t len) {
    global.parse->dir = i;

    if (!host_cb_store_dir) {
        iwarn("Host callback store_dir not set.");
        return;
    }
    host_cb_store_dir(host_parse_results, i, len);
}
const char *rh_parse_parse_get_dir () {
    return global.parse->dir;
}

void rh_parse_parse_store_cdata_int (int i) {
    int int_str_length = int_length(i);
    char *str = malloc((int_str_length + 1) * sizeof(char));
    sprintf(str, "%d", i);
    global.parse->cdata = str;
    if (!host_cb_store_cdata) {
        iwarn("Host callback store_cdata not set.");
        return;
    }
    host_cb_store_cdata(host_parse_results, str, int_str_length + 1);
}

#if TEST
int main(int argc, char **argv) {
    rh_parse_init();
    rh_parse_set_input("ls 2 T");
    rh_parse_start();
}
#endif


