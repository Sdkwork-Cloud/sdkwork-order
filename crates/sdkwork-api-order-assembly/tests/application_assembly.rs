use sdkwork_api_order_assembly::ApiAssembly;
use sdkwork_web_core::RouteAuth;

#[test]
fn application_manifest_exports_complete_recharge_surface() {
    let manifest = ApiAssembly::app_route_manifest();
    let packages = manifest
        .match_route("GET", "/app/v3/api/recharges/packages")
        .expect("recharge packages route must be exported by the assembly");
    let create_order = manifest
        .match_route("POST", "/app/v3/api/recharges/orders")
        .expect("recharge order route must be exported by the assembly");

    assert_eq!(RouteAuth::DualToken, packages.auth);
    assert_eq!("recharges.packages.list", packages.operation_id);
    assert_eq!(RouteAuth::DualToken, create_order.auth);
    assert_eq!("recharges.orders.create", create_order.operation_id);
    assert_eq!(41, manifest.routes().len());
}
