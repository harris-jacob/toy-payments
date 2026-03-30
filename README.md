# Payment Challenge

## Running

Application runs via:

```
cargo run -- transactions.csv
```

Run tests with:

```
cargo run test
```

## Extra Assumptions
I made a couple extra assumptions along the way, here they are:

- Account's available funds can go negative during disputes.
- No operations can be performed on frozen accounts.
- For chargebacks, disputes and resolves, the client id must match the client
ID in the recorded transaction, if it does not, the operation is ignored.
- Only deposits can be disputed. Since the spec mentions 'available funds
should decrease', I assume this means withdrawals do not get disputed. This is
nice because it means we don't really need to save a record of withdrawals.


## Correctness and testing
Some tests are included for the fundamental domain operations. Given more time,
I would like to have tested the `state.rs` file, particularly each transaction type
and the various edge cases e.g.:

- Missing Client ID (creates new client).
- Chargeback/resolve, non disputed transaction are ignored.
- Disputing an already disputed transaction does nothing.
- Resolve/chargeback/dispute where the client_id does not match recorded transaction
does nothing.
- operations on frozen accounts do nothing.

RE correctness, where possible I have leveraged the type system but there are some cases
I would have liked to do a little more (given more time). I don't like the fact that
in the 'domain' layer, even transaction types without an 'amount' are represented with 
a zero amount (its pretty sloppy -- and writen this way because I just started with
withdrawal/deposits). Given more time i'd refactor to something like:

```rust
enum Transaction {
    Withdrawal(TransactionWithAmount),
    Deposit(TransactionWithAmount),
    Dispute(Transaction)
    ..
}

struct TransactionWithAmount {
    id: TxId,
    client_id: ClientId,
    amount: Amount
}

struct Transaction {
    id: TxId,
    client_id: ClientId,
}

```

Similarly, I don't like how the responsibility of account freezing is delegated
to the `state.rs` file. I would prefer to take a similar approach here:

```rust
enum Account {
    Frozen(AccountState),
    Active(AccountState)
}

impl Account {
    fn do_something(&mut self) {
        match account {
            Frozen => return,
            Active(state) => {
                ..
            }
        }
    }

}

```

## Safety and Robustness

The program does two things with errors: 

- Swallows them and continues, these are cases where we don't want to interrupt the flow
of the program. Normally we'd do something like log these kinds of errors but since
the program is using the std out, i've just opted to swallow them for this exercise.

- Panics, I've reserved this for when a known business invariant is violated, 
ideal these errors would be protected by tests in the application logic.

## Efficiency

I am streaming the csv file via a reader and there are no rogue collects going on
so it should be able to handle large data files. We're limited by how many transaction
and account records we can store in memory but these are pretty light, as mentioned above,
only deposits are stored.

RE CSVs coming from thousand of concurrent TCP streams. The main problem 
would be concurrent state access. There are a few approaches here:

- Client level locks (might get pretty messy).
- Sharding by client (if we really needed multiple 'engine' threads).
- Actor model, streams are read and submit transactions to an engine thread via
channels (my preferred approach).
