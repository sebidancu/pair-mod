# Replace the following with your own values (You need to run the script once to get the contract address)
OWNER="/Users/Sebi/ElrondSC/Sc-CrowdFunding/walletTest.pem"
ADDRESS="erd1qqqqqqqqqqqqqpgqr2qwr8rnrhe4qrxhnk4ez77l3mumd9s4j0wqxw0f33"
# erd1qqqqqqqqqqqqqpgqr2qwr8rnrhe4qrxhnk4ez77l3mumd9s4j0wqxw0f33 - new ride / usdc
# erd1qqqqqqqqqqqqqpgq493rr8r9863zugs3e27jmn5x8wh2zqhrj0wq2ma4rg
# erd1qqqqqqqqqqqqqpgqezgle0q7l05lkqsle6vdkmwymp4dntjsj0wqjt0wpp - ride wegld
#OWNER="erd1...xxx"
# Place your keystore file in the same directory as this script and replace the following with the name of the file
# Optionally, you can also put your password in the .passfile in the same directory as this script (if not, you will be prompted for the password)
WASM_PATH="/Users/Sebi/ElrondSC/OrderBook/pair-mod/output/order-book-pair.wasm"
PRIVATE_KEY="/Users/Sebi/ElrondSC/Sc-CrowdFunding/walletTest.pem"
PROXY=https://devnet-api.multiversx.com
CHAIN_ID=D
DEPLOY_GAS="80000000"

# Standard deploy command. Provide any constructor arguments as needed (e.g deploy 12 TOKEN-123456). Numbers are automatically scaled to 18 decimals. (e.g. 12 -> 12000000000000000000)
deploy() {
# Arguments: 
    ARG_0=str:RIDE-6e4c49  # 0: first_token_id (TokenIdentifier)
    ARG_1=str:WEGLD-d7c6bb  # 1: second_token_id (TokenIdentifier)

    mxpy --verbose contract deploy --project=${PROJECT} --recall-nonce --pem=${OWNER} \
          --gas-limit=${DEPLOY_GAS} \
          --outfile="deploy.interaction.json" --send --proxy=${PROXY} --chain=D --arguments ${ARG_0} ${ARG_1} || return

    echo "Deployed contract at the address written above."
    echo "Pleade update the ADDRESS variable in this script with the address of the deployed contract, then run 'source interaction.sh' to update the environment variables."
}

upgradeSC() {
    ARG_0=str:RIDE-6e4c49  # 0: first_token_id (TokenIdentifier)
    ARG_1=str:WEGLD-d7c6bb  # 1: second_token_id (TokenIdentifier)
    
    mxpy --verbose contract upgrade ${ADDRESS} --recall-nonce \
        --bytecode=${WASM_PATH} \
        --pem=${PRIVATE_KEY} \
        --gas-limit=100000000 \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
         --arguments ${ARG_0} ${ARG_1} \
        --send || return
}


# All contract endpoints are available as functions. Provide any arguments as needed (e.g transfer 12 TOKEN-123)

setProvider() {
    # Arguments: 
    ARG_0=0xe9e5d24305ef5bded3d3dab5320ab9e48a5aa61bd8ed208542452289c7bf93dc
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=10000000 \
        --function="setProvider" \
        --proxy=${PROXY} --chain=D \
        --arguments $ARG_0 \
        --send
}


ARGUMENT_2=0xe9e5d24305ef5bded3d3dab5320ab9e48a5aa61bd8ed208542452289c7bf93dc

RIDE=str:RIDE-6e4c49 
WEGLD=str:WEGLD-d7c6bb
USDC=str:USDC-8d4068
VALUE=0x4563918244f40000 #5wegld
VALUE_USDC=50000000 #50 $
ARGUMENT_1=30000000000000000000

createBuyOrder() {
    VALUE_WANT=100000000000000000000 #RIDE
    VALUE_SEND=100000000 #USDC 

    user_address="$(mxpy wallet pem-address $OWNER)"
    method_name=str:createBuyOrder
    destination_address=$ADDRESS
    mxpy --verbose contract call $ADDRESS --recall-nonce \
        --pem=${OWNER} \
        --gas-limit=20000000 \
        --proxy=${PROXY} --chain=D \
        --function="ESDTTransfer" \
        --arguments $USDC $VALUE_SEND \
                    $method_name \
                    $VALUE_WANT \
        --send || return
}
# ESDTTransfer@WEGLD@4563918244F40000@createBuyOrder@ARGUMENT_1@ARGUMENT_2@ARGUMENT_3@ARGUMENT_4@ARGUMENT_5@ARGUMENT_6

createSellOrder3() {
    VALUE_WANT=81000000 #USDC
    VALUE_SEND=60000000000000000000 #RIDE

    user_address="$(mxpy wallet pem-address $OWNER)"
    method_name=str:createSellOrder
    destination_address=$ADDRESS
    mxpy --verbose contract call $ADDRESS --recall-nonce \
        --pem=${OWNER} \
        --gas-limit=20000000 \
        --proxy=${PROXY} --chain=D \
        --function="ESDTTransfer" \
        --arguments $RIDE $VALUE_SEND \
                    $method_name \
                    $VALUE_WANT \
        --send || return

}

createSellOrder2() {
    VALUE_WANT=26600000 #USDC
    VALUE_SEND=20000000000000000000 #RIDE

    user_address="$(mxpy wallet pem-address $OWNER)"
    method_name=str:createSellOrder
    destination_address=$ADDRESS
    mxpy --verbose contract call $ADDRESS --recall-nonce \
        --pem=${OWNER} \
        --gas-limit=20000000 \
        --proxy=${PROXY} --chain=D \
        --function="ESDTTransfer" \
        --arguments $RIDE $VALUE_SEND \
                    $method_name \
                    $VALUE_WANT \
        --send || return

}

createSellOrder1() {
    VALUE_WANT=6500000 #USDC
    VALUE_SEND=5000000000000000000 #RIDE

    user_address="$(mxpy wallet pem-address $OWNER)"
    method_name=str:createSellOrder
    destination_address=$ADDRESS
    mxpy --verbose contract call $ADDRESS --recall-nonce \
        --pem=${OWNER} \
        --gas-limit=20000000 \
        --proxy=${PROXY} --chain=D \
        --function="ESDTTransfer" \
        --arguments $RIDE $VALUE_SEND \
                    $method_name \
                    $VALUE_WANT \
        --send || return

}

fund() {
    VALUE_WANT=100000000 #USDC
    VALUE_SEND=60000000000000000000 #RIDE

    user_address="$(mxpy wallet pem-address $OWNER)"
    method_name=str:fund
    destination_address=$ADDRESS
    mxpy --verbose contract call $ADDRESS --recall-nonce \
        --pem=${OWNER} \
        --gas-limit=20000000 \
        --proxy=${PROXY} --chain=D \
        --function="ESDTTransfer" \
        --arguments $USDC $VALUE_WANT \
                    $method_name \
        --send || return

}

withdraw(){
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=10000000 \
        --function="withdraw" \
        --proxy=${PROXY} --chain=D \
        --send
}

matchOrders() {
# Arguments: 
    ARG_0=1
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=10000000 \
        --function="matchOrders" \
        --proxy=${PROXY} --chain=D \
        --arguments 3 123 124 125 126 \
        --send

}
cancel_all_orders(){
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER} --gas-limit=10000000 \
        --function="cancelAllOrders" \
        --proxy=${PROXY} --chain=D \
        --send
}

changeFirstToken(){
    user_address="$(mxpy wallet pem-address $OWNER)"
    method_name=str:createSellOrder
    destination_address=$ADDRESS
    mxpy --verbose contract call $ADDRESS --recall-nonce \
        --pem=${OWNER} \
        --gas-limit=20000000 \
        --proxy=${PROXY} --chain=D \
        --function="changeFirstToken" \
        --arguments $RIDE \
        --send || return

}

changeSecondToken(){
    user_address="$(mxpy wallet pem-address $OWNER)"
    method_name=str:createSellOrder
    destination_address=$ADDRESS
    mxpy --verbose contract call $ADDRESS --recall-nonce \
        --pem=${OWNER} \
        --gas-limit=20000000 \
        --proxy=${PROXY} --chain=D \
        --function="changeSecondToken" \
        --arguments $USDC \
        --send || return

}

cancelOrders() {
# Arguments: 
ARG_0=${1}  # 0: order_ids (variadic<u64>)
    mxpy contract call ${ADDRESS} \
        --recall-nonce ${PRIVATE_KEY} --gas-limit=500000000 --proxy=${PROXY} --chain=${CHAIN_ID} --send \
        --function "cancelOrders" \
        --arguments 8 9

}

cancelAllOrders() {
    mxpy contract call ${ADDRESS} \
        --recall-nonce ${PRIVATE_KEY} --gas-limit=500000000 --proxy=${PROXY} --chain=${CHAIN_ID} --send \
        --function "cancelAllOrders" 
}

freeOrders() {
# Arguments: 
ARG_0=${1}  # 0: order_ids (variadic<u64>)
    mxpy contract call ${ADDRESS} \
        --recall-nonce ${PRIVATE_KEY} --gas-limit=500000000 --proxy=${PROXY} --chain=${CHAIN_ID} --send \
        --function "freeOrders" \
        --arguments ${ARG_0} 

}

startGlobalOperation() {
    mxpy contract call ${ADDRESS} \
        --recall-nonce ${PRIVATE_KEY} --gas-limit=500000000 --proxy=${PROXY} --chain=${CHAIN_ID} --send \
        --function "startGlobalOperation" 
}

stopGlobalOperation() {
    mxpy contract call ${ADDRESS} \
        --recall-nonce ${PRIVATE_KEY} --gas-limit=500000000 --proxy=${PROXY} --chain=${CHAIN_ID} --send \
        --function "stopGlobalOperation" 
}

# All contract views. Provide arguments as needed (e.g balanceOf 0x1234567890123456789012345678901234567890)

getAddressOrderIds() {
# Arguments: 
ARG_0=${0}  # 0: address (Address)
    mxpy contract query ${ADDRESS} \
        --function "getAddressOrderIds" \
        --proxy=${PROXY} \
         --arguments ${ARG_0} 

}

getOrderIdCounter() {
    mxpy contract query ${ADDRESS} \
        --function "getOrderIdCounter" \
        --proxy=${PROXY} 
}

getOrderById() {
# Arguments: 
ARG_0=33  #$(echo "scale=0; (${1}*10^18)/1" | bc -l)  # 0: id (u64)
    mxpy contract query ${ADDRESS} \
        --function "getOrderById" \
        --proxy=${PROXY} \
         --arguments ${ARG_0} 

}

getFirstTokenId() {
    mxpy contract query ${ADDRESS} \
        --function "getFirstTokenId" \
        --proxy=${PROXY} 
}

getSecondTokenId() {
    mxpy contract query ${ADDRESS} \
        --function "getSecondTokenId" \
        --proxy=${PROXY} 
}

getAllOrders(){
    mxpy contract query ${ADDRESS} \
    --function "getAllOrders" \
    --proxy=${PROXY} 

}

getFlag(){
    mxpy contract query ${ADDRESS} \
    --function "getFlag" \
    --proxy=${PROXY} 
}

getOutputOrderId() {
ARG_0=25  #$(echo "scale=0; (${1}*10^18)/1" | bc -l)  # 0: id (u64)
    mxpy contract query ${ADDRESS} \
        --function "getOutputOrderId" \
        --proxy=${PROXY} \
         --arguments ${ARG_0} 

}

getInputOrderId(){
ARG_0=25  #$(echo "scale=0; (${1}*10^18)/1" | bc -l)  # 0: id (u64)
    mxpy contract query ${ADDRESS} \
        --function "getInputOrderId" \
        --proxy=${PROXY} \
         --arguments ${ARG_0} 

}

getPricePerToken(){
    mxpy contract query ${ADDRESS} \
    --function "getPricePerToken" \
    --proxy=${PROXY} 
}
# 1_350_000

createBuyLimitOrder1(){
    VALUE_WANT=5000000000000000000 #RIDE
    VALUE_SEND=6500000 #USDC 

    user_address="$(mxpy wallet pem-address $OWNER)"
    method_name=str:createBuyOrder
    destination_address=$ADDRESS
    mxpy --verbose contract call $ADDRESS --recall-nonce \
        --pem=${OWNER} \
        --gas-limit=20000000 \
        --proxy=${PROXY} --chain=D \
        --function="ESDTTransfer" \
        --arguments $USDC $VALUE_SEND \
                    $method_name \
                    $VALUE_WANT \
        --send || return

}

createBuyLimitOrder2(){
    VALUE_WANT=15000000000000000000 #RIDE
    VALUE_SEND=18000000 #USDC 

    user_address="$(mxpy wallet pem-address $OWNER)"
    method_name=str:createBuyOrder
    destination_address=$ADDRESS
    mxpy --verbose contract call $ADDRESS --recall-nonce \
        --pem=${OWNER} \
        --gas-limit=20000000 \
        --proxy=${PROXY} --chain=D \
        --function="ESDTTransfer" \
        --arguments $USDC $VALUE_SEND \
                    $method_name \
                    $VALUE_WANT \
        --send || return

}

createBuyLimitOrder3(){
    VALUE_WANT=25000000000000000000 #RIDE
    VALUE_SEND=27500000 #USDC 

    user_address="$(mxpy wallet pem-address $OWNER)"
    method_name=str:createBuyOrder
    destination_address=$ADDRESS
    mxpy --verbose contract call $ADDRESS --recall-nonce \
        --pem=${OWNER} \
        --gas-limit=20000000 \
        --proxy=${PROXY} --chain=D \
        --function="ESDTTransfer" \
        --arguments $USDC $VALUE_SEND \
                    $method_name \
                    $VALUE_WANT \
        --send || return

}

createSellMarketOrder() {
    VALUE_WANT=0 #USDC
    VALUE_SEND=22000000000000000000 #RIDE

    user_address="$(mxpy wallet pem-address $OWNER)"
    method_name=str:createSellOrder
    destination_address=$ADDRESS
    mxpy --verbose contract call $ADDRESS --recall-nonce \
        --pem=${OWNER} \
        --gas-limit=20000000 \
        --proxy=${PROXY} --chain=D \
        --function="ESDTTransfer" \
        --arguments $RIDE $VALUE_SEND \
                    $method_name \
                    $VALUE_WANT \
        --send || return

}