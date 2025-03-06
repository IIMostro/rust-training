use std::sync::Arc;
use std::time::Duration;

use pingora_core::Error;
use pingora_core::prelude::{background_service, HttpPeer, Server};
use pingora_core::server::configuration::Opt;
use pingora_http::RequestHeader;
use pingora_load_balancing::{health_check, LoadBalancer};
use pingora_load_balancing::prelude::RoundRobin;
use pingora_proxy::{http_proxy_service, ProxyHttp, Session};
use structopt::StructOpt;
use tracing::info;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub struct Lb{
    lb: Arc<LoadBalancer<RoundRobin>>
}


#[async_trait::async_trait]
impl ProxyHttp for Lb {
    type CTX = ();

    fn new_ctx(&self) -> Self::CTX {
        ()
    }



    async fn upstream_peer(&self, session: &mut Session, ctx: &mut Self::CTX) -> pingora_core::Result<Box<HttpPeer>> {
        let upstream = self.lb.select(b"", 256).unwrap();
        info!("upstream peer is: {:?}", upstream);
        Ok(Box::new(HttpPeer::new(upstream, false, String::new())))
    }

    async fn request_filter(&self, _session: &mut Session, _ctx: &mut Self::CTX) -> pingora_core::Result<bool>
    where
        Self::CTX: Send + Sync
    {
        info!("request_filter scope");
        Ok(false)
    }

    async fn upstream_request_filter(&self, _session: &mut Session, _upstream_request: &mut RequestHeader, _ctx: &mut Self::CTX) -> pingora_core::Result<()>
    where
        Self::CTX: Send + Sync,
    {
        info!("upstream_request_filter scope");
        _upstream_request.insert_header("Host", "pay.closeli.cn").unwrap();
        Ok(())
    }

    async fn logging(&self, _session: &mut Session, _e: Option<&Error>, _ctx: &mut Self::CTX)
    where
        Self::CTX: Send + Sync
    {
        info!("logging scope");
    }
}

fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .init();
    let opt = Some(Opt::from_args());
    info!("opt: {:?}", opt);
    let mut server = Server::new(opt).unwrap();
    info!("configuration: {:?}", &server.configuration);
    server.bootstrap();
    let upstreams = LoadBalancer::try_from_iter(["pay.closeli.cn:80"]).unwrap();
    let mut lb = http_proxy_service(&server.configuration, Lb{lb : Arc::new(upstreams)});
    lb.add_tcp("0.0.0.0:6188");
    server.add_service(lb);
    server.run_forever();
}

#[cfg(test)]
mod tests {
    use pingora_load_balancing::LoadBalancer;
    use pingora_load_balancing::prelude::RoundRobin;

    #[test]
    pub fn test_from_iter_should_work(){
        let upstreams:LoadBalancer<RoundRobin> = LoadBalancer::try_from_iter(["pay.closeli.cn:80", "pay.stg.closeli.cn:80"]).unwrap();
        println!("{:?}", upstreams.backends().get_backend());
        let a= upstreams.select(b"", 256).unwrap();
        println!("hello world!")
    }
}

