#!/bin/bash

printf ">> sending transaction to ztestsapling1kg3u0y7szv6509732at34alct46cyn0g26kppgf2a7h5tpqxldtwm7cmhf8rqmhgtmpakcz5mdv with memo 68656c6c6f207a63617368 via the mixnet"
op_id=$(zcash-cli -testnet -rpcport=8080 z_sendmany utest1pk856zmyl5ccn56mx68v7j0v0lx2xc9kgq9ug5aq8wa6fm70d2mtyn929js696tlp54zpayz7js92jp6gr56249pm39gf4jrw5ahtsfq49dqsfjy65jyjdeh9ketj3lunpe2vdzgs4a9vkyqlfg9896yt9wqymq2dkzsmw5cykx6rk4f84vjj7fxqgse69jklkpumj3g6wvt7ucxxjr '[{"memo":"68656c6c6f207a63617368","address": "ztestsapling1kg3u0y7szv6509732at34alct46cyn0g26kppgf2a7h5tpqxldtwm7cmhf8rqmhgtmpakcz5mdv","amount":0.000001}]')

sleep 1

op_array="[\"$op_id\"]"
tx=$(zcash-cli -testnet -rpcport=8080 z_getoperationstatus $op_array)
printf "\nstatus: $tx"

txid=$(echo "$tx" | jq -r '.[0].result.txid')
printf "\ntransaction overview: $(zcash-cli -testnet -rpcport=8080 z_viewtransaction $txid)"
