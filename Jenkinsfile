pipeline {
    agent { label 'docker' }
    environment {
        CARGO_HOME = "${WORKSPACE}/.cargo"
    }
    stages {
        stage('Check') {
            parallel {
                stage('Check tessa4d') {
                    agent {
                        docker {
                            // Reusing bevy-build for make and rustup components
                            image 'gitea.okaymonkey.com/cicd/bevy-build:latest'
                            alwaysPull true
                            reuseNode true
                        }
                    }
                    steps {
                        sh 'make check-tessa'
                    }
                }
                stage('Check tessa4d-bevy') {
                    agent {
                        docker {
                            image 'gitea.okaymonkey.com/cicd/bevy-build:latest'
                            alwaysPull true
                            reuseNode true
                        }
                    }
                    steps {
                        sh 'make check-bevy'
                    }
                }
            }
        }
        stage('Integration Tests') {
            parallel {
                stage('itest tessa4d-bevy') {
                    agent {
                        docker {
                            image 'gitea.okaymonkey.com/cicd/bevy-itest:latest'
                            alwaysPull true
                            reuseNode true
                        }
                    }
                    steps {
                        sh 'xvfb-run -s "-screen 0 1280x1024x24" make itest-bevy'
                    }
                }
            }
        }
    }
    post {
        always {
            archiveArtifacts artifacts: 'tessa4d-bevy/assets/screenshots/**/*.png', fingerprint: true
        }
    }
}
