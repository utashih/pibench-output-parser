use regex::{self, Regex};
use std::error;

#[derive(PartialOrd, PartialEq, Debug)]
pub enum KeyDistribution {
    UNIFORM = 0,
    ZIPFAN = 1,
}

#[derive(PartialEq, PartialOrd, Debug)]
pub struct BenchmarkOptions {
    sampling: i32,
    latency: f32,
    key_size: i32,
    value_size: i32,
    random_seed: i32,
    read: f32,
    insert: f32,
    update: f32,
    delete: f32,
    scan: f32,
    key_distribution: KeyDistribution,
    records: i32,
    operations: i32,
    threads: i32,
}

impl BenchmarkOptions {
    pub fn from_text(text: &str) -> Result<BenchmarkOptions, Box<dyn error::Error>> {
        let re = Regex::new(
            r"# Records: (?P<records>\d+)\s*
           \s*# Operations: (?P<operations>\d+)\s*
           \s*# Threads: (?P<threads>\d+)\s*
           \s*Sampling: (?P<sampling>\d+) ms\s*
           \s*Latency: (?P<latency>\d*\.?\d*)\s*
           .*
           \s*Key size: (?P<key_size>\d+)\s*
           \s*Value size: (?P<value_size>\d+)\s*
           \s*Random seed: (?P<random_seed>\d+)\s*
           \s*Key distribution: (?P<key_distribution>\w+)\s*
           \s*Scan size: (?P<scan_size>\d+)\s*
           .*
           \s*Read: (?P<read>\d*\.?\d*)\s*
           \s*Insert: (?P<insert>\d*\.?\d*)\s*
           \s*Update: (?P<update>\d*\.?\d*)\s*
           \s*Delete: (?P<delete>\d*\.?\d*)\s*
           \s*Scan: (?P<scan>\d*\.?\d*)\s*",
        )
        .unwrap();
        let caps = re.captures(text).unwrap();

        Ok(BenchmarkOptions {
            sampling: caps["sampling"].parse::<i32>()?,
            records: caps["records"].parse::<i32>()?,
            threads: caps["threads"].parse::<i32>()?,
            operations: caps["operations"].parse::<i32>()?,
            latency: caps["latency"].parse::<f32>()?,
            key_size: caps["key_size"].parse::<i32>()?,
            key_distribution: KeyDistribution::UNIFORM,
            value_size: caps["value_size"].parse::<i32>()?,
            random_seed: caps["random_seed"].parse::<i32>()?,
            read: caps["read"].parse::<f32>()?,
            insert: caps["insert"].parse::<f32>()?,
            delete: caps["delete"].parse::<f32>()?,
            update: caps["update"].parse::<f32>()?,
            scan: caps["scan"].parse::<f32>()?,
        })
    }
}

#[derive(Eq, PartialEq, PartialOrd, Debug)]
pub struct LatencyResults {
    min: i32,
    p_50: i32,
    p_90: i32,
    p_99: i32,
    p_99_9: i32,
    p_99_99: i32,
    p_99_999: i32,
    max: i32,
}

impl LatencyResults {
    pub fn from_text(text: &str) -> Result<Option<LatencyResults>, Box<dyn error::Error>> {
        let re = Regex::new(
            r"Latencies .*
            \s*min: (?P<min>\d+)\s*
            \s*50%: (?P<p_50>\d+)\s*
            \s*90%: (?P<p_90>\d+)\s*
            \s*99%: (?P<p_99>\d+)\s*
            \s*99.9%: (?P<p_99_9>\d+)\s*
            \s*99.99%: (?P<p_99_99>\d+)\s*
            \s*99.999%: (?P<p_99_999>\d+)\s*
            \s*max: (?P<max>\d+)\s*",
        )?;
        let caps = match re.captures(text) {
            Some(caps) => caps,
            None => return Ok(None),
        };
        Ok(Some(LatencyResults {
            min: caps["min"].parse::<i32>()?,
            p_50: caps["p_50"].parse::<i32>()?,
            p_90: caps["p_90"].parse::<i32>()?,
            p_99: caps["p_99"].parse::<i32>()?,
            p_99_9: caps["p_99_9"].parse::<i32>()?,
            p_99_99: caps["p_99_99"].parse::<i32>()?,
            p_99_999: caps["p_99_999"].parse::<i32>()?,
            max: caps["max"].parse::<i32>()?,
        }))
    }
}

#[derive(PartialEq, PartialOrd, Debug)]
pub struct BenchmarkResults {
    load_time: f32,
    run_time: f32,
    throughput: f32,
    l3_misses: Option<u64>,
    dram_reads: Option<u64>,
    dram_writes: Option<u64>,
    nvm_reads: Option<u64>,
    nvm_writes: Option<u64>,
    latency: Option<LatencyResults>,
}

impl BenchmarkResults {
    pub fn from_text(text: &str) -> Result<BenchmarkResults, Box<dyn error::Error>> {
        const FLOATING_REGEX: &str = "[+\\-]?(?:0|[1-9]\\d*)(?:\\.\\d*)?(?:[eE][+\\-]?\\d+)?";
        let regex_raw = format!(
            r"Load time: (?P<load_time>{floating}) milliseconds\s*
            \s*Run time: (?P<run_time>{floating}) milliseconds\s*
            \s*Throughput: (?P<throughput>{floating}) ops/s\s*
            .*
            \s*L3 misses: (?P<l3_misses>\d+)\s*
            \s*DRAM Reads \(bytes\): (?P<dram_reads>\d+)\s*
            \s*DRAM Writes \(bytes\): (?P<dram_writes>\d+)\s*
            \s*NVM Reads \(bytes\): (?P<nvm_reads>\d+)\s*
            \s*NVM Writes \(bytes\): (?P<nvm_writes>\d+)\s*",
            floating = FLOATING_REGEX
        );
        let re = Regex::new(&regex_raw)?;
        let caps = re.captures(text).unwrap();

        let latency_results = LatencyResults::from_text(text)?;

        Ok(BenchmarkResults {
            load_time: caps["load_time"].parse::<f32>()?,
            run_time: caps["run_time"].parse::<f32>()?,
            throughput: caps["throughput"].parse::<f32>()?,
            l3_misses: Some(caps["l3_misses"].parse::<u64>()?),
            dram_reads: Some(caps["dram_reads"].parse::<u64>()?),
            dram_writes: Some(caps["dram_writes"].parse::<u64>()?),
            nvm_reads: Some(caps["nvm_reads"].parse::<u64>()?),
            nvm_writes: Some(caps["nvm_writes"].parse::<u64>()?),
            latency: latency_results,
        })
    }
}

pub struct PiBenchData {
    benchmark_options: BenchmarkOptions,
    benchmark_results: BenchmarkResults,
}

pub fn parse(input: &str) -> PiBenchData {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn parse_benchmark_results() {
        let sample_string = "Overview:
                                    Load time: 90801.3 milliseconds
                                    Run time: 79192.3672 milliseconds
                                    Throughput: 126274.7969 ops/s
                                PCM Metrics:
                                    L3 misses: 133342466
                                    DRAM Reads (bytes): 4197345472
                                    DRAM Writes (bytes): 3685394624
                                    NVM Reads (bytes): 60347831872
                                    NVM Writes (bytes): 11408209856
                                Samples:
                                ";
        let gt = BenchmarkResults {
            load_time: 90801.3,
            run_time: 79192.3672,
            throughput: 126274.7969,
            l3_misses: Some(133342466),
            dram_reads: Some(4197345472),
            dram_writes: Some(3685394624),
            nvm_reads: Some(60347831872),
            nvm_writes: Some(11408209856),
            latency: None,
        };
        let result = BenchmarkResults::from_text(sample_string);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), gt);
    }

    #[test]
    fn parse_latency_results() {
        let sample_string = "Latencies (998141 operations observed):
                                    min: 882
                                    50%: 7481
                                    90%: 9121
                                    99%: 43233
                                    99.9%: 51150
                                    99.99%: 69460
                                    99.999%: 16985300
                                    max: 22247728
                                ";
        let gt = LatencyResults {
            min: 882,
            p_50: 7481,
            p_90: 9121,
            p_99: 43233,
            p_99_9: 51150,
            p_99_99: 69460,
            p_99_999: 16985300,
            max: 22247728,
        };
        let latency = LatencyResults::from_text(sample_string);
        assert!(latency.is_ok());
        assert_eq!(latency.unwrap().unwrap(), gt);
    }

    #[test]
    fn parse_benchmark_options() {
        let sample_string = "Benchmark Options:
                                    Target: /home/hao/coding/bztree/release/libbztree_pibench_wrapper.so
                                    # Records: 10000000
                                    # Operations: 10000000
                                    # Threads: 1
                                    Sampling: 1000 ms
                                    Latency: 0.1
                                    Key prefix: 
                                    Key size: 8
                                    Value size: 8
                                    Random seed: 1729
                                    Key distribution: UNIFORM
                                    Scan size: 100
                                    Operations ratio:
                                        Read: 0.2
                                        Insert: 0.8
                                        Update: 0
                                        Delete: 0
                                        Scan: 0
                                creating new tree on pool.
                                ";
        let gt = BenchmarkOptions {
            records: 10000000,
            operations: 10000000,
            threads: 1,
            sampling: 1000,
            latency: 0.1,
            key_size: 8,
            value_size: 8,
            random_seed: 1729,
            key_distribution: KeyDistribution::UNIFORM,
            scan: 0.,
            read: 0.2,
            insert: 0.8,
            update: 0.,
            delete: 0.,
        };
        let options = BenchmarkOptions::from_text(sample_string);
        assert!(options.is_ok());
        assert_eq!(options.unwrap(), gt);
    }
}
