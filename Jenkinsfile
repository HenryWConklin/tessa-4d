pipeline {
    agent {
        dockerfile { filename 'Dockerfile.jenkins' }
    }
    stages {
        stage('CI') {
            parallel {
                stage('Run Checks') {
                    steps {
                        sh '"./run-checks.sh"'
                    }
                }
                stage('Integration Tests') {
                    // Xvfb allows running X without an acutal display
                    // so that we can take screenshots and test rendering in itests.
                    steps {
                        sh 'pulseaudio & xvfb-run -s "-screen 0 1280x1024x24" ./run-itests.sh'
                    }
                }
            }
        }
    }
    post {
        always {
            archiveArtifacts artifacts: 'itest/tessa4d-gdext/tests/screenshots/*.png', fingerprint: true
        }
    }
}
