// let mqtt_settings = Arc::clone(&configs).read().await.mqtt.clone();

// let db_clone = db.clone();

// let server_status: Arc<RwLock<Option<i64>>> = Arc::new(RwLock::new(None));
// let mqtt_client = MqttClient::new(mqtt_settings, server_status.clone()).await;
// let mqtt_client_ptr = Arc::new(mqtt_client.clone());

// let clients_workspaces: ClientsWorkspaces = Arc::new(RwLock::new(HashMap::new()));

// mqtt_client
//     .subscribe("entry/registration", QoS::AtLeastOnce)
//     .await
//     .ok();
// mqtt_client
//     .add_topic_handler("entry/registration", move |payload| {
//         let db_clone = db_clone.clone();
//         let mqtt_client_clone = Arc::clone(&mqtt_client_ptr);

//         tokio::spawn(async move {
//             let handler = handle_device_registration(&payload, db_clone).await;
//             if let Err(e) = handler {
//                 error!("error on entry data: {}", e);
//                 let r = mqtt_client_clone
//                     .publish("entry/reports", &e.as_str(), QoS::AtLeastOnce, true)
//                     .await;
//                 if let Err(err) = r {
//                     error!("error publishing: {}", err);
//                 }
//             }
//         });
//     })
//     .await;

// let db_clone = db.clone();
// let mqtt_client_ptr = Arc::new(mqtt_client.clone());
// mqtt_client
//     .subscribe("entry/heartbeat", QoS::AtLeastOnce)
//     .await
//     .ok();
// mqtt_client.add_topic_handler("entry/heartbeat", move |payload| {
//     let db_clone = db_clone.clone();
//     let mqtt_client_clone = Arc::clone(&mqtt_client_ptr);

//     tokio::spawn(async move {
//         let handler = read_device_heartbeat(&payload, db_clone).await;
//         if let Err(e) = handler {
//             error!("error on entry data: {}", e);
//             let r = mqtt_client_clone
//                 .publish("entry/reports", &e.as_str(), QoS::AtLeastOnce, true)
//                 .await;
//             if let Err(err) = r {
//                 error!("error publishing: {}", err);
//             }
//         }
//     });
// }).await;

// let db_clone = db.clone();
// let mqtt_client_ptr = Arc::new(mqtt_client.clone());
// mqtt_client
//     .subscribe("entry/heartbeat/threads", QoS::AtLeastOnce)
//     .await
//     .ok();
// mqtt_client.add_topic_handler("entry/heartbeat/threads", move |payload| {
//     let db_clone = db_clone.clone();
//     let mqtt_client_clone = Arc::clone(&mqtt_client_ptr);

//     tokio::spawn(async move {
//         let handler = read_device_threads_heartbeat(&payload, db_clone).await;
//         if let Err(e) = handler {
//             error!("error on entry data: {}", e);
//             let r = mqtt_client_clone
//                 .publish("entry/reports", &e.as_str(), QoS::AtLeastOnce, true)
//                 .await;
//             if let Err(err) = r {
//                 error!("error publishing: {}", err);
//             }
//         }
//     });
// }).await;

// let db_clone = db.clone();
// let mqtt_client_ptr = Arc::new(mqtt_client.clone());
// mqtt_client
//     .subscribe("entry/data", QoS::AtLeastOnce)
//     .await
//     .ok();
// mqtt_client
//     .add_topic_handler("entry/data", move |payload| {
//         let db_clone = db_clone.clone();
//         let mqtt_client_clone = Arc::clone(&mqtt_client_ptr);
//         let ws_conns = ws_connections_clone.clone();

//         tokio::spawn(async move {
//             let handler = handle_entry_data(db_clone, payload.as_str(), ws_conns).await;
//             if let Err(e) = handler {
//                 error!("error on entry data: {}", e);
//                 let r = mqtt_client_clone
//                     .publish("entry/reports", &e.as_str(), QoS::AtLeastOnce, true)
//                     .await;
//                 if let Err(err) = r {
//                     error!("error publishing: {}", err);
//                 }
//             }
//         });
//     })
//     .await;

// let db_clone = db.clone();
// let mqtt_client_ptr = Arc::new(mqtt_client.clone());
// mqtt_client
//     .subscribe("sensors/realtime/data", QoS::AtLeastOnce)
//     .await
//     .ok();
// mqtt_client
//     .add_topic_handler("sensors/realtime/data", move |payload| {
//         let db_clone = db_clone.clone();
//         let ws_conns = ws_connections_clone.clone();
//         let mqtt_client_clone = Arc::clone(&mqtt_client_ptr);

//         tokio::spawn(async move {
//             let handler = handle_real_time_data(db_clone, &payload, ws_conns).await;
//             if let Err(e) = handler {
//                 error!("error on entry data: {}", e);
//                 let r = mqtt_client_clone
//                     .publish("entry/reports", &e.as_str(), QoS::AtLeastOnce, true)
//                     .await;
//                 if let Err(err) = r {
//                     error!("error publishing: {}", err);
//                 }
//             }
//         });
//     })
//     .await;

// let mqtt_client_ptr = Arc::new(mqtt_client.clone());
// let db_clone = db.clone();
