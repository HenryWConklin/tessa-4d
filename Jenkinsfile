pipeline {
    agent {
        docker { image 'rust:1.68.1-slim' }
    }
    stages {
        stage('Test') {
            steps {
                sh 'cargo test'
            }
        }
        stage('Clippy') {
            steps {
                sh 'rustup component add clippy'
                sh 'cargo clippy'
            }
        }
        stage('Format') {
            steps {
                sh 'rustup component add rustfmt'
                sh 'cargo fmt --check'
            }
        }
    }
}