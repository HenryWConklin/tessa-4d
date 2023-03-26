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
    }
}