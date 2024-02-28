use async_trait::async_trait;
use pingora_core::{server::Server, services::background::background_service, upstreams::peer::HttpPeer, Result};
use pingora_load_balancing::{health_check, selection::RoundRobin, LoadBalancer};
use pingora_proxy::{ProxyHttp, Session};
use std::sync::Arc;

pub struct LB(Arc<LoadBalancer<RoundRobin>>);

#[async_trait]
impl ProxyHttp for LB {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let upstream = self
            .0
            .select(b"", 256) // hash doesn't matter
            .unwrap();

        let peer = Box::new(HttpPeer::new(upstream, false, "0.0.0.0".to_string()));
        Ok(peer)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut pingora_http::RequestHeader,
        _ctx: &mut Self::CTX
    ) -> Result<()> {
        upstream_request.insert_header("Host", "0.0.0.0").unwrap();
        Ok(())
    }
}

// RUST_LOG=INFO cargo run --example load_balancer
fn main() {
    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();

    let mut upstreams = LoadBalancer::try_from_iter(["0.0.0.0:8080"]).unwrap();

    let hc = health_check::TcpHealthCheck::new();
    upstreams.set_health_check(hc);
    upstreams.health_check_frequency = None;
    let background = background_service("health check", upstreams);
    let upstreams = background.task();

    let mut lb = pingora_proxy::http_proxy_service(&my_server.configuration, LB(upstreams));
    lb.add_tcp("0.0.0.0:9999");

    my_server.add_service(lb);
    my_server.add_service(background);
    my_server.run_forever();
}
