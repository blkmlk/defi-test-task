# Task Testing Guide

This repository contains four tasks, each with its own test suite. Follow the instructions below to test each task.

## Prerequisites

Before running the tests, you must start a **local Solana validator**. This is required for all tasks to interact with
the Solana blockchain environment.

To start a local validator, run:

```bash
solana-test-validator
```

## How to Test

1. **Task 1**
    - Navigate to the task1 directory:
      ```bash
      cd task1
      ```
    - Run the tests:
      ```bash
      make test
      ```

2. **Task 2**
    - Navigate to the task2 directory:
      ```bash
      cd task2
      ```
    - Run the tests:
      ```bash
      make test
      ```

3. **Task 3**
    - Navigate to the task3 directory:
      ```bash
      cd task3
      ```
    - Run the tests:
      ```bash
      make test
      ```

4. **Task 4**
    - Navigate to the task4 directory:
      ```bash
      cd task4
      ```
    - Run the tests:
      ```bash
      make test
      ```

Make sure `make` is installed on your system before running these commands. Each task directory should contain a
`Makefile` with a `test` target defined.