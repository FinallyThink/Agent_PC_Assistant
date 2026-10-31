use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
pub struct HealthResult {
    status: String,
    message: String,
}

pub fn local_endpoint(address: &str) -> Result<reqwest::Url, String> {
    let url = reqwest::Url::parse(address).map_err(|_| "地址格式不正确")?;
    if url.scheme() != "http"
        || !matches!(url.host_str(), Some("127.0.0.1") | Some("[::1]"))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("仅支持 http://127.0.0.1:端口 或 http://[::1]:端口，不接受路径或凭据".into());
    }
    Ok(url)
}

#[tauri::command]
pub async fn check_connection(address: String) -> Result<HealthResult, String> {
    let url = local_endpoint(&address)?
        .join("health")
        .map_err(|_| "地址无效")?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|_| "无法初始化 HTTP 客户端")?;
    match client.get(url).send().await {
        Ok(response) => Ok(HealthResult {
            status: if response.status().is_success() {
                "online"
            } else {
                "error"
            }
            .into(),
            message: format!("GET /health · HTTP {}", response.status().as_u16()),
        }),
        Err(error) => Ok(HealthResult {
            status: if error.is_timeout() {
                "unknown"
            } else {
                "offline"
            }
            .into(),
            message: if error.is_timeout() {
                "健康检查超时（5 秒），服务状态未知"
            } else {
                "无法连接本地服务，请检查端口和服务进程"
            }
            .into(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_remote_credentials_and_paths() {
        for address in [
            "https://127.0.0.1:9881",
            "http://example.com",
            "http://127.0.0.1.evil.com",
            "http://user:secret@127.0.0.1",
            "http://127.0.0.1/a",
            "http://127.0.0.1/?token=x",
        ] {
            assert!(local_endpoint(address).is_err(), "{address}");
        }
    }
    #[test]
    fn accepts_loopback() {
        assert_eq!(
            local_endpoint("http://127.0.0.1:9881").unwrap().as_str(),
            "http://127.0.0.1:9881/"
        );
        assert!(local_endpoint("http://[::1]:9881").is_ok());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };
    fn mock(status: &str) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        let response =
            format!("HTTP/1.1 {status}\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}");
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut data = [0; 2048];
            let count = stream.read(&mut data).unwrap();
            assert!(String::from_utf8_lossy(&data[..count]).starts_with("GET /health "));
            stream.write_all(response.as_bytes()).unwrap();
        });
        (address, handle)
    }
    #[tokio::test]
    async fn independent_services_report_their_actual_status() {
        let (first, a) = mock("200 OK");
        let (second, b) = mock("503 Service Unavailable");
        assert_eq!(check_connection(first).await.unwrap().status, "online");
        assert_eq!(check_connection(second).await.unwrap().status, "error");
        a.join().unwrap();
        b.join().unwrap();
    }
    #[tokio::test]
    async fn offline_is_not_online() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        drop(listener);
        assert_eq!(check_connection(address).await.unwrap().status, "offline");
    }
}

#[cfg(test)]
mod timeout_tests {
    use super::*;
    use std::{net::TcpListener, thread};
    #[tokio::test]
    async fn timeout_reports_unknown() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        let handle = thread::spawn(move || {
            let (_stream, _) = listener.accept().unwrap();
            thread::sleep(Duration::from_secs(6));
        });
        assert_eq!(check_connection(address).await.unwrap().status, "unknown");
        handle.join().unwrap();
    }
}

#[derive(Serialize)]
pub struct CallResult {
    pub status: String,
    pub data: Option<serde_json::Value>,
    pub message: String,
}

#[tauri::command]
pub async fn call_capability(address: String, capability: String) -> Result<CallResult, String> {
    let mut url = local_endpoint(&address)?;
    let path = match capability.as_str() {
        "service.health" => "/health",
        "service.capabilities" => "/capabilities",
        _ => return Err("不支持的能力，仅允许 service.health 和 service.capabilities".into()),
    };
    url.set_path(path);
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|_| "客户端初始化失败")?;
    let result = async {
        let mut response = client
            .get(url)
            .send()
            .await
            .map_err(|e| if e.is_timeout() { "TIMEOUT" } else { "OFFLINE" }.to_string())?;
        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status().as_u16()));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|e| {
            if e.is_timeout() {
                "TIMEOUT"
            } else {
                "READ_ERROR"
            }
            .to_string()
        })? {
            if bytes.len() + chunk.len() > 262144 {
                return Err("响应超过 256 KiB 限制".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        serde_json::from_slice::<serde_json::Value>(&bytes)
            .map_err(|_| "服务未返回有效 JSON".into())
    }
    .await;
    Ok(match result {
        Ok(data) => CallResult {
            status: "succeeded".into(),
            data: Some(data),
            message: "只读调用完成".into(),
        },
        Err(message) => CallResult {
            status: if message == "TIMEOUT" {
                "unknown"
            } else {
                "failed"
            }
            .into(),
            data: None,
            message,
        },
    })
}

#[cfg(test)]
mod capability_tests {
    use super::*;
    #[tokio::test]
    async fn rejects_unknown_before_network() {
        assert!(
            call_capability("http://127.0.0.1:1".into(), "shell.execute".into())
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn reads_json() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        let handle = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut buffer = [0; 1024];
            let count = socket.read(&mut buffer).unwrap();
            assert!(String::from_utf8_lossy(&buffer[..count]).starts_with("GET /capabilities "));
            socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n[]")
                .unwrap();
        });
        let result = call_capability(address, "service.capabilities".into())
            .await
            .unwrap();
        assert_eq!(result.status, "succeeded");
        assert_eq!(result.data, Some(serde_json::json!([])));
        handle.join().unwrap();
    }
}
