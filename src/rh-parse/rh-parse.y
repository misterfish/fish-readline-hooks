%{

//#define YYDEBUG 1

// --- talk about the parsing.
//#define DO_GIVE_ME_THAT

// --- strnlen.
#define _POSIX_C_SOURCE 200809L

#include <stdio.h>
#include <stdlib.h>

// --- va macros / vprintf.
#include <stdarg.h>

// --- strlen.
#include <string.h>

// --- yylex, rh_parse_store/get routines.
#include "rh-parse.h"

void yyerror (char const *);

#ifdef DO_GIVE_ME_THAT
void jibber_jabber(const char *fmt, ...) {
    int fmt_new_l = strnlen(fmt, 100);
    char *fmt_new = malloc((fmt_new_l + 1 + 1) * sizeof(char));
    snprintf(fmt_new, fmt_new_l + 1 + 1, "%s\n", fmt);
    va_list va;
    va_start(va, fmt);
    vprintf(fmt_new, va);
    va_end(va);
}
#else
void jibber_jabber(const char *fmt, ...) {
}
#endif

void done() {
    jibber_jabber("»»»»»» done.");
    int i;
    short s;
    const char *c;
    if (i = rh_parse_parse_get_num()) {
        jibber_jabber("»»» num: %d", i);
    }
    if (c = rh_parse_parse_get_cdata()) {
        jibber_jabber("»»» cdata: %s", c);
    }
    if (c = rh_parse_parse_get_command()) {
        jibber_jabber("»»» command: %s", c);
    }
    if (c = rh_parse_parse_get_dir()) {
        jibber_jabber("»»» dir: %s", c);
    }
}

%}

%union {
  char const *strval;
  int intval;
}


/*
precedence 1: terminal symbols.
precedence 2: "Each rule gets its precedence from the last terminal symbol mentioned in
 the components"
precedence 2.5: you can specify the precedence of a rule.
precedence finally: compare precedence of rule with precedence with that of
lookahead token. If latter, shift. If former, reduce.

shift/reduce conflicts are resolved in favor of shifting.
 */

/*
%precedence T_NUM
%precedence T_CDATA
*/

%precedence T_LS_T

/*
Name in <> corresponds to field in union.
Then comes the token.
Last field is an optional alias in "".
*/

%token <strval> T_CDATA
%token <intval> T_NUM

%token T_LS_T
%token T_LS_TR
%token T_LS_L
%token T_LS_LR
%token T_LS_Z

/* custom: */
%token T_LS_A1
%token T_LS_A2


%token T_END

/* Just to make stacking of rules easier (never returned) */
%token T_NULL

%%

/* latest is highest precedence
 */

input:
  %empty {
}| cdata_phrase T_END {
    done();
}| cdata_phrase '=' command T_END {
    done();
}| '=' command T_END {
    done();
}
;

cdata_phrase:
   cdata_phrase T_CDATA {
        rh_parse_parse_store_cdata($2, strlen($2) + 1);
        jibber_jabber("»»» cdata_phrase = cdata_phrase T_CDATA |%s|", $2);
}| cdata_phrase T_NUM {
        rh_parse_parse_store_cdata_int($2);
        jibber_jabber("»»» cdata_phrase = cdata_phrase T_NUM   |%d|", $2);

        // coupled XX
}| cdata_phrase T_LS_T {
        rh_parse_parse_store_cdata("t", 2);
        jibber_jabber("»»» cdata_phrase = cdata_phrase T_LS_T   ||");
}| cdata_phrase T_LS_TR {
        rh_parse_parse_store_cdata("tr", 3);
}| cdata_phrase T_LS_L {
        rh_parse_parse_store_cdata("l", 2);
}| cdata_phrase T_LS_LR {
        rh_parse_parse_store_cdata("lr", 3);

        // custom:
}| cdata_phrase T_LS_A1 {
        rh_parse_parse_store_cdata("ag", 3);
}| cdata_phrase T_LS_A2 {
        rh_parse_parse_store_cdata("af", 3);

}| cdata_phrase T_LS_Z {
        rh_parse_parse_store_cdata("z", 2);
}| T_CDATA {
        rh_parse_parse_store_cdata($1, strlen($1) + 1);
        jibber_jabber("»»» cdata_phrase = T_CDATA |%s|", $1);
}| T_NUM {
        rh_parse_parse_store_cdata_int($1);
        jibber_jabber("»»» cdata_phrase = T_NUM |%d|", $1);

        // coupled XX
}| T_LS_T {
        rh_parse_parse_store_cdata("t", 2);
        jibber_jabber("»»» cdata_phrase = T_LS_T ||");
}| T_LS_TR {
        rh_parse_parse_store_cdata("tr", 3);
}| T_LS_L {
        rh_parse_parse_store_cdata("l", 2);
}| T_LS_LR {
        rh_parse_parse_store_cdata("lr", 3);

        // custom:
}| T_LS_A1 {
        rh_parse_parse_store_cdata("ag", 3);
}| T_LS_A2 {
        rh_parse_parse_store_cdata("af", 3);

}| T_LS_Z {
        rh_parse_parse_store_cdata("z", 2);
};

command:
  %empty {
        // alias t
        rh_parse_parse_store_command("t", 2);
}| dir num T_LS_L {
        rh_parse_parse_store_command("l", 2);
        jibber_jabber("»»» command = ls-l");
}| dir num T_LS_LR {
        rh_parse_parse_store_command("lr", 3);
}| dir num T_LS_T {
        rh_parse_parse_store_command("t", 2);
}| dir num T_LS_TR {
        rh_parse_parse_store_command("tr", 3);
/* causes shift/reduce conflict.
}| dir_non_empty num_non_empty {
        rh_parse_parse_store_command("t", 2);
*/
}| dir_non_empty {
        rh_parse_parse_store_command("t", 2);

        // custom:
}| T_LS_A1 {
        rh_parse_parse_store_command("ag", 3);
}| T_LS_A2 {
        rh_parse_parse_store_command("af", 3);

}| T_LS_Z {
        rh_parse_parse_store_command("z", 2);
};

dir:
  %empty {
}| dir_non_empty;

dir_non_empty:
   T_CDATA {
        rh_parse_parse_store_dir($1, strlen($1) + 1);
};

num:
  %empty {
}| num_non_empty;

num_non_empty:
   T_NUM {
        rh_parse_parse_store_num($1);
};

%%

#include <stdio.h>

int yydebug = 1;

/* Called by yyparse on error.  */
void yyerror (char const *s) {
    fprintf (stderr, "»»» Bad news young person: %s\n", s);
}
