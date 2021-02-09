This repo contains a multithreaded simulation of tasks along with a `warp` web server. It serves up requests at `http://localhost:8080/metrics`. These metrics are "pulled"/scraped by the prometheus server.

To test,

- run prometheus at the folder root
- run the rust simulation, `cargo run`

You can now navigate to `http://localhost:9090/graph` to view the prometheus console.

## Some sample queries:

90% quantile for the past two minues for the time bucket :

```
histogram_quantile(0.99, (rate(thread_times_bucket[2m])))
```

Average latency in 10 minute intervals

```
sum without (instance)(rate(thread_times_sum[10m]))
/
sum without (instance)(rate(thread_times_count[10m]))
```
