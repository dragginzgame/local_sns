#!/bin/bash

local_sns_replica_url() {
    if [ -n "${LOCAL_SNS_REPLICA_URL:-}" ]; then
        echo "${LOCAL_SNS_REPLICA_URL%/}"
    elif [ -n "${ICP_REPLICA_URL:-}" ]; then
        echo "${ICP_REPLICA_URL%/}"
    else
        echo "http://127.0.0.1:8000"
    fi
}

local_sns_replica_status_url() {
    echo "$(local_sns_replica_url)/api/v2/status"
}

local_sns_replica_is_reachable() {
    if ! command -v curl >/dev/null 2>&1; then
        return 1
    fi

    curl --max-time 2 -fsS "$(local_sns_replica_status_url)" >/dev/null 2>&1
}

ensure_icp_network() {
    if icp network ping >/dev/null 2>&1; then
        return 0
    fi

    if local_sns_replica_is_reachable; then
        print_warning "Using external ICP replica at $(local_sns_replica_url)"
        return 0
    fi

    print_error "No reachable ICP replica found."
    print_info "Start this project's network with: icp network start -d"
    print_info "Or point to an existing replica with: LOCAL_SNS_REPLICA_URL=http://127.0.0.1:8000"
    return 1
}
