//
// Created by karolgruszka on 27.08.2020.
//

#ifndef CLOX_VALUE_H
#define CLOX_VALUE_H


#include "../common.h"

typedef double Value;

typedef struct {
    int capacity;
    int count;
    Value *values;
} ValueArray;

void initValueArray(ValueArray *array);

int writeValueArray(ValueArray *array, Value value);

void freeValueArray(ValueArray *array);

void printValue(Value value);

#endif //CLOX_VALUE_H
