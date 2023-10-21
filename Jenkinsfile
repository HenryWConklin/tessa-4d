pipeline {
    agent {
        dockerfile { filename 'Dockerfile.jenkins' }
    }
    stages {
        stage('Run Checks') {
            steps {
                sh './run_checks.sh'
            }
        }
    }
}
