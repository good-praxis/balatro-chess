pipeline {
    agent any

    stages {
        stage('Build') {
            steps {
                sh "~/.cargo/bin/cargo build"
            }
        }
        stage('Test') {
            steps {
                sh "~/.cargo/bin/cargo test"
            }
        }
        stage('Clippy') {
            steps {
                sh "~/.cargo/bin/cargo +nightly clippy --all"
            }
        }
        stage('Rustfmt') {
            steps {
                // The build will fail if rustfmt thinks any changes are
                // required.
                sh "~/.cargo/bin/cargo +nightly fmt --all"
            }
        }
        stage('Benchmark') {
            steps {
                sh "~/.cargo/bin/cargo bench --bench stepping_pieces"
                sh "~/.cargo/bin/cargo bench --bench sliding_pieces"
                sh "~/.cargo/bin/cargo bench --bench make_unmake_no_capture"
                sh "~/.cargo/bin/cargo bench --bench make_unmake_capture"
                sh "~/.cargo/bin/cargo bench --bench board_to_string"
                sh "~/.cargo/bin/cargo bench --bench search_depth_1"
                sh "~/.cargo/bin/cargo bench --bench search_depth_3"
            }
        }
        stage('Miri') {
            steps {
                sh "~/.cargo/bin/cargo miri setup"
                sh "MIRIFLAGS='-Zmiri-disable-stacked-borrows' ~/.cargo/bin/cargo miri test"
            }
        }
    }
}
