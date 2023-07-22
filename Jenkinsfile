pipeline {
    agent {
        dockerfile { filename 'Dockerfile.jenkins' }
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
