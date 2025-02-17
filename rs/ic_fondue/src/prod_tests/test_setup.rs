use super::bootstrap::{MaliciousNodes, NodeVms};
use super::driver_setup::DriverContext;
use crate::ic_manager::{FarmInfo, IcEndpoint, IcHandle, IcSubnet, RuntimeDescriptor};
use ic_base_types::NodeId;
use ic_prep_lib::{
    initialized_subnet::InitializedSubnet, internet_computer::InitializedIc, node::InitializedNode,
    prep_state_directory::IcPrepStateDir,
};
use std::time::Instant;

pub fn create_ic_handle(
    ctx: &DriverContext,
    init_ic: &InitializedIc,
    vm_nodes: &NodeVms,
    mal_beh: &MaliciousNodes,
) -> IcHandle {
    let mut public_api_endpoints = vec![];
    let mut malicious_public_api_endpoints = vec![];

    vm_nodes.iter().for_each(|(node_id, vm)| {
        let (subnet, node) = if let Some((subnet, node)) = node_id_to_subnet(init_ic, *node_id) {
            (Some(subnet), node)
        } else {
            (
                None,
                init_ic
                    .unassigned_nodes
                    .get(node_id)
                    .expect("node not initialized"),
            )
        };
        let url = node
            .node_config
            .public_api
            .first()
            .expect("No public API endpoint specified")
            .clone()
            .into();
        let metrics_url = node
            .node_config
            .prometheus_metrics
            .first()
            .expect("No metrics address specified")
            .clone()
            .into();

        let endpoint = IcEndpoint {
            runtime_descriptor: RuntimeDescriptor::Vm(FarmInfo {
                group_name: vm.group_name.clone(),
                vm_name: vm.name.clone(),
                url: ctx.farm.base_url.clone(),
            }),
            url,
            metrics_url: Some(metrics_url),
            subnet: subnet.map(|s| IcSubnet {
                id: s.subnet_id,
                type_of: s.subnet_config.subnet_type,
            }),
            is_root_subnet: subnet.map(|s| s.subnet_index) == Some(0),
            started_at: Instant::now(),
            ssh_key_pairs: ctx.authorized_ssh_accounts.clone(),
        };

        if mal_beh.contains_key(node_id) {
            malicious_public_api_endpoints.push(endpoint);
        } else {
            public_api_endpoints.push(endpoint);
        }
    });

    let ic_prep_working_dir = Some(IcPrepStateDir::new(&init_ic.target_dir));
    IcHandle {
        public_api_endpoints,
        malicious_public_api_endpoints,
        ic_prep_working_dir,
    }
}

fn node_id_to_subnet(
    init_ic: &InitializedIc,
    node_id: NodeId,
) -> Option<(&InitializedSubnet, &InitializedNode)> {
    init_ic
        .initialized_topology
        .values()
        .filter_map(|subnet| {
            subnet
                .initialized_nodes
                .values()
                .find(|n| n.node_id == node_id)
                .map(|n| (subnet, n))
        })
        .next()
}
