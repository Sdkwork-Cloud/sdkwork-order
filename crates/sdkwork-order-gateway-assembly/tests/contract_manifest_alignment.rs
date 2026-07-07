use sdkwork_order_service::order_service_contract;
use sdkwork_routes_order_app_api::http_route_manifest::app_route_manifest;
use sdkwork_routes_order_backend_api::http_route_manifest::backend_route_manifest;

#[test]
fn order_service_contract_operations_are_declared_on_http_manifest() {
    let contract = order_service_contract();
    let manifest_ops = app_route_manifest()
        .routes()
        .iter()
        .chain(backend_route_manifest().routes().iter())
        .map(|route| route.operation_id)
        .collect::<Vec<_>>();

    for operation_id in contract
        .write_commands
        .iter()
        .chain(contract.read_queries.iter())
    {
        assert!(
            manifest_ops.iter().any(|candidate| candidate == operation_id),
            "contract operation {operation_id} must be declared on app or backend HTTP manifest"
        );
    }
}
