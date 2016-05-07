#ifndef __BISON_CLIENT_H

/*
typedef enum {
    RH_COMMAND_INVALID,
    RH_COMMAND_LS_L,
    RH_COMMAND_LS_LR,
    RH_COMMAND_LS_T,
    RH_COMMAND_LS_TR
} rh_command_type;
*/

// --- export for the generated parser to hook into.
int yylex (void);

// --- called from parser during events.
void rh_parse_parse_store_num (int);
void rh_parse_parse_store_command (const char *, size_t);
void rh_parse_parse_store_cdata (const char *, size_t);
void rh_parse_parse_store_cdata_int (int);
void rh_parse_parse_store_dir (const char *, size_t);

int rh_parse_parse_get_num ();
const char *rh_parse_parse_get_command ();
const char *rh_parse_parse_get_cdata ();
const char *rh_parse_parse_get_dir ();

/* FFI */

// --- pointer to a function which takes a void* and x and returns y.
//
// all of them take a a void* first: that's the handle to the rust results
// object.

typedef void (*rust_callback__int__void)(void*, int);
//typedef void (*rust_callback__rh_command_type__void)(int);
typedef void (*rust_callback__const_char_star_and_size_t__void)(void*, const char *, size_t);

void rh_parse_register_cb_store_num (rust_callback__int__void);
void rh_parse_register_cb_store_command (rust_callback__const_char_star_and_size_t__void);
void rh_parse_register_cb_store_dir (rust_callback__const_char_star_and_size_t__void);
void rh_parse_register_cb_store_cdata (rust_callback__const_char_star_and_size_t__void);

#define __BISON_CLIENT_H
#endif
