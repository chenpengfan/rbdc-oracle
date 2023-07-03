create table T_STUDENT
(
    
    ID_CARD  NUMBER not null
        constraint "T_STUDENT_pk"
            primary key,
    NAME     VARCHAR2(100),
    AGE      NUMBER(3),
    BIRTHDAY DATE,
    SEX      NUMBER(1),
    SCORE    NUMBER(8)
);

create table T_STUDENT_PROFILE
(
    ID_CARD  NUMBER not null
        constraint "T_STUDENT_PROFILE_pk"
            primary key,
    PHOTO BLOB,
    RESUME CLOB
);